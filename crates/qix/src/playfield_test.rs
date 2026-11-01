use eframe::egui::{self, Pos2, Rect, Shape, Vec2, pos2, vec2};
use qix_core::tuning::MARKER_STEP_TICKS;
use qix_core::tuning::{BOARD_H, BOARD_W};
use qix_core::{
    Board, Cell, Dir, DrawKind, Game, HighScore, HighScores, Initials, InputState, Qix, Rng,
};

use crate::bindings::KeyBindings;
use crate::palette;
use crate::playfield::{
    Placement, Scene, hud_units, letterbox, playfield_units, show, to_screen, window_units,
};
use crate::render::board::CellRun;
use crate::render::hud;
use crate::render::screen::{self, Screen, paused, title};

/// Seed of every game the tests play.
const SEED: u64 = 1;

/// High scores on the title screen painted by
/// [`a_screen_adds_its_heading_each_row_and_each_line_over_the_playfield`].
const TITLE_SCORES: u32 = 3;

/// Tolerance for comparing screen coordinates, in points.
const EPSILON: f32 = 1e-3;

/// Size of the window area the painting tests run in.
const PAINTED_AREA: Vec2 = vec2(800.0, 600.0);

/// Window areas to fit the playfield into: wide, tall, exact and tiny.
const AREAS: [Vec2; 5] = [
    vec2(1920.0, 1080.0),
    vec2(600.0, 1000.0),
    vec2(800.0, 600.0),
    vec2(333.0, 250.0),
    vec2(1.0, 1.0),
];

/// The aspect ratio of the playfield with the HUD above it.
fn aspect() -> f32 {
    f32::from(BOARD_W) / (f32::from(BOARD_H) + hud::HEIGHT)
}

fn assert_near(actual: Pos2, expected: Pos2) {
    assert!(
        (actual - expected).length() < EPSILON,
        "{actual:?} != {expected:?}"
    );
}

fn area(size: Vec2) -> Rect {
    Rect::from_min_size(pos2(10.0, 20.0), size)
}

#[test]
fn the_playfield_keeps_its_aspect_ratio_at_any_size() {
    for size in AREAS {
        let field = letterbox(area(size));
        let ratio = field.width() / field.height();
        assert!((ratio - aspect()).abs() < EPSILON, "{size:?}: {ratio}");
    }
}

#[test]
fn the_playfield_fills_one_axis_and_stays_centred_inside() {
    for size in AREAS {
        let available = area(size);
        let field = letterbox(available);
        let fills_width = (field.width() - available.width()).abs() < EPSILON;
        let fills_height = (field.height() - available.height()).abs() < EPSILON;
        assert!(fills_width || fills_height, "{size:?}: {field:?}");
        assert!(available.expand(EPSILON).contains_rect(field), "{size:?}");
        assert_near(field.center(), available.center());
    }
}

#[test]
fn a_wide_window_letterboxes_left_and_right() {
    let (width, height) = (1600.0, 600.0);
    let field = letterbox(Rect::from_min_size(Pos2::ZERO, vec2(width, height)));
    let margin = (width - height * aspect()) / 2.0;
    assert_near(field.min, pos2(margin, 0.0));
    assert_near(field.max, pos2(width - margin, height));
}

#[test]
fn the_hud_sits_above_the_playfield_as_wide_as_it() {
    let hud = hud_units();
    let field = playfield_units();
    assert_eq!(hud.left_bottom(), field.left_top());
    assert_eq!(hud.right_bottom(), field.right_top());
    assert_eq!(hud.height(), hud::HEIGHT);
    assert_eq!(window_units(), hud.union(field));
}

#[test]
fn an_empty_area_gives_an_empty_playfield() {
    let field = letterbox(Rect::from_min_size(Pos2::ZERO, Vec2::ZERO));
    assert_eq!(field.size(), Vec2::ZERO);
}

#[test]
fn the_hud_and_playfield_corners_map_to_the_letterbox_corners() {
    let available = area(vec2(1000.0, 500.0));
    let transform = to_screen(available);
    let letterboxed = letterbox(available);
    let hud = transform.transform_rect(hud_units());
    let field = transform.transform_rect(playfield_units());
    assert_near(hud.min, letterboxed.min);
    assert_near(field.max, letterboxed.max);
    assert_near(hud.left_bottom(), field.left_top());
}

/// The HUD's strip on screen in the painting tests.
fn hud_strip() -> Rect {
    to_screen(area(PAINTED_AREA)).transform_rect(hud_units())
}

/// Runs one headless egui frame showing `game`'s playfield and returns
/// the shapes it painted outside the HUD, after checking that they
/// tessellate.
fn painted(game: &Game, overlay: bool) -> Vec<Shape> {
    painted_with(game, None, overlay)
}

/// [`painted`] with `screen` centred over the playfield.
fn painted_with(game: &Game, screen: Option<&Screen>, overlay: bool) -> Vec<Shape> {
    let hud = hud::hud(game, 0);
    painted_scene(&Scene {
        game,
        hud: &hud,
        claim: None,
        screen: screen.map(|screen| (screen, Placement::Centred)),
        overlay,
    })
}

/// The shapes of one headless egui frame showing `scene`, outside the
/// HUD, after checking that they tessellate.
fn painted_scene(scene: &Scene) -> Vec<Shape> {
    let strip = hud_strip();
    let shapes = painted_all(scene);
    let in_hud = |shape: &Shape| strip.contains_rect(shape.visual_bounding_rect());
    shapes.into_iter().filter(|shape| !in_hud(shape)).collect()
}

/// [`painted_scene`] of `game` with `screen` placed as `placement`.
fn painted_placed(game: &Game, screen: &Screen, placement: Placement) -> Vec<Shape> {
    let hud = hud::hud(game, 0);
    painted_scene(&Scene {
        game,
        hud: &hud,
        claim: None,
        screen: Some((screen, placement)),
        overlay: false,
    })
}

/// Every shape of one headless egui frame showing `scene`, after
/// checking that they tessellate.
fn painted_all(scene: &Scene) -> Vec<Shape> {
    let ctx = egui::Context::default();
    let input = egui::RawInput {
        screen_rect: Some(area(PAINTED_AREA)),
        ..egui::RawInput::default()
    };
    let mut output = ctx.run_ui(input, |ui| show(ui, scene));
    // No renderer uploads the font atlas here; dropping it unapplied panics.
    output.textures_delta.clear();
    let shapes = output.shapes.iter().map(|clipped| clipped.shape.clone());
    let shapes = shapes.collect();
    let primitives = ctx.tessellate(output.shapes, output.pixels_per_point);
    assert!(!primitives.is_empty(), "nothing to render");
    shapes
}

fn count(shapes: &[Shape], kind: fn(&Shape) -> bool) -> usize {
    shapes.iter().filter(|&shape| kind(shape)).count()
}

#[test]
fn the_playfield_paints_the_claimed_cells_border_and_marker() {
    let mut board = Board::new(BOARD_W, BOARD_H);
    board.set_cell(0, 0, Cell::Claimed(DrawKind::Slow));
    let mut rng = Rng::new(SEED);
    let qix = Qix::spawn(&board, &mut rng);
    let mut game = Game::with_board(board, vec![qix], Vec::new(), rng);
    game.tick(&InputState::default());
    let shapes = painted(&game, false);
    let meshes = count(&shapes, |s| matches!(s, Shape::Mesh(m) if !m.is_empty()));
    let segments = count(
        &shapes,
        |s| matches!(s, Shape::LineSegment { stroke, .. } if stroke.color == palette::BORDER),
    );
    assert_eq!(meshes, 2, "the claimed cells and the marker: {shapes:?}");
    // The four sides, cut short by the claimed corner, and its two edges.
    assert_eq!(segments, 6, "{shapes:?}");
    assert_eq!(count(&shapes, |s| matches!(s, Shape::Text(_))), 0);
}

#[test]
fn the_overlay_adds_its_text() {
    let shapes = painted(&Game::new(Rng::new(SEED)), true);
    assert_eq!(count(&shapes, |s| matches!(s, Shape::Text(_))), 1);
}

#[test]
fn a_screen_adds_its_heading_each_row_and_each_line_over_the_playfield() {
    let game = Game::new(Rng::new(SEED));
    let table = (1..=TITLE_SCORES)
        .map(|score| HighScore {
            name: Initials::FIRST,
            score,
        })
        .collect::<Vec<_>>();
    let screen = title(&HighScores::from(table), &KeyBindings::default());
    assert_eq!(screen.table.len(), usize::try_from(TITLE_SCORES).unwrap());
    let shapes = painted_with(&game, Some(&screen), false);
    let texts = count(&shapes, |s| matches!(s, Shape::Text(_)));
    assert_eq!(texts, 1 + screen.table.len() + screen.lines.len());
    let field = painted(&game, false);
    assert_eq!(
        shapes[..field.len()],
        field[..],
        "the playfield is not painted under the screen"
    );
}

/// The text of each text shape of `shapes`, in painting order.
fn texts(shapes: &[Shape]) -> Vec<String> {
    shapes
        .iter()
        .filter_map(|shape| match shape {
            Shape::Text(text) => Some(text.galley.text().to_owned()),
            _ => None,
        })
        .collect()
}

#[test]
fn the_title_paints_the_high_scores_between_the_heading_and_the_controls() {
    let table = HighScores::from(vec![HighScore {
        name: Initials::FIRST,
        score: 1,
    }]);
    let screen = title(&table, &KeyBindings::default());

    let shapes = painted_with(&Game::new(Rng::new(SEED)), Some(&screen), false);

    assert_eq!(
        texts(&shapes)[..3],
        ["QIX", "1. AAA  1", "Up/Down/Left/Right: move"]
    );
}

#[test]
fn the_overlay_stays_above_a_screen() {
    let shapes = painted_with(&Game::new(Rng::new(SEED)), Some(&paused()), true);
    let last_text = shapes.iter().rev().find(|s| matches!(s, Shape::Text(_)));
    assert!(
        matches!(last_text, Some(Shape::Text(text)) if text.galley.text().starts_with("claimed")),
        "{last_text:?}"
    );
}

#[test]
fn a_stix_being_drawn_is_painted_as_a_line() {
    let mut game = Game::new(Rng::new(SEED));
    let drawing_up = InputState {
        dir: Some(Dir::Up),
        fast: true,
        ..InputState::default()
    };
    for _ in 0..MARKER_STEP_TICKS {
        game.tick(&drawing_up);
    }
    assert!(game.stix().is_some(), "the marker did not start a stix");
    let shapes = painted(&game, false);
    assert_eq!(
        count(&shapes, |s| matches!(s, Shape::Path(p) if !p.closed)),
        1
    );
}

#[test]
fn the_hud_is_painted_in_its_strip() {
    let game = Game::new(Rng::new(SEED));
    let strip = hud_strip();
    let hud = hud::hud(&game, 0);
    let shapes = painted_all(&Scene {
        game: &game,
        hud: &hud,
        claim: None,
        screen: None,
        overlay: false,
    });
    let in_hud = |shape: &&Shape| strip.contains_rect(shape.visual_bounding_rect());
    let in_hud: Vec<_> = shapes.iter().filter(in_hud).cloned().collect();
    let texts = texts(&in_hud);
    assert!(texts.contains(&"SCORE".to_owned()), "{texts:?}");
    assert!(texts.contains(&"0.0% / 75%".to_owned()), "{texts:?}");
}

/// Whether `shape` is a mesh with a vertex of `colour`.
fn mesh_in(shape: &Shape, colour: egui::Color32) -> bool {
    matches!(shape, Shape::Mesh(mesh) if mesh.vertices.iter().any(|v| v.color == colour))
}

#[test]
fn a_claim_being_animated_is_painted_over_the_cells_under_the_marker() {
    let game = Game::new(Rng::new(SEED));
    let hud = hud::hud(&game, 0);
    let runs = [CellRun {
        row: 0,
        cols: 0..BOARD_W,
        kind: DrawKind::Fast,
    }];
    let shapes = painted_scene(&Scene {
        game: &game,
        hud: &hud,
        claim: Some((&runs, 0.0)),
        screen: None,
        overlay: false,
    });
    let flash = shapes
        .iter()
        .position(|shape| mesh_in(shape, palette::CLAIM_FLASH));
    let marker = shapes
        .iter()
        .position(|shape| mesh_in(shape, palette::MARKER_OUTLINE));
    let flash = flash.expect("the claim is painted");
    assert!(flash < marker.expect("the marker is painted"));
    assert_eq!(painted(&game, false).len() + 1, shapes.len());
}

/// Whether `shape` is the dimming of the playfield under a screen.
fn is_dim(shape: &Shape) -> bool {
    matches!(shape, Shape::Rect(rect) if rect.fill == palette::SCREEN_DIM)
}

#[test]
fn a_centred_screen_dims_the_playfield() {
    let shapes = painted_placed(&Game::new(Rng::new(SEED)), &paused(), Placement::Centred);
    assert_eq!(count(&shapes, is_dim), 1);
}

#[test]
fn a_banner_leaves_the_playfield_undimmed_and_sits_at_its_top() {
    let banner = screen::demo();
    let shapes = painted_placed(&Game::new(Rng::new(SEED)), &banner, Placement::Banner);
    assert_eq!(count(&shapes, is_dim), 0);
    assert_eq!(texts(&shapes).len(), 1 + banner.lines.len());
    let field = to_screen(area(PAINTED_AREA)).transform_rect(playfield_units());
    for shape in shapes.iter().filter(|s| matches!(s, Shape::Text(_))) {
        let bounds = shape.visual_bounding_rect();
        assert!(field.contains_rect(bounds), "{bounds:?} off {field:?}");
        assert!(
            bounds.max.y < field.center().y,
            "{bounds:?} below the middle"
        );
    }
}
