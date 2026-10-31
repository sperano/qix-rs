use eframe::egui::{self, Pos2, Rect, Shape, Vec2, pos2, vec2};
use qix_core::tuning::MARKER_STEP_TICKS;
use qix_core::tuning::{BOARD_H, BOARD_W};
use qix_core::{Board, Cell, Dir, DrawKind, Game, InputState, Qix, Rng};

use crate::palette;
use crate::playfield::{letterbox, playfield_units, show, to_screen};

/// Seed of every game the tests play.
const SEED: u64 = 1;

/// Tolerance for comparing screen coordinates, in points.
const EPSILON: f32 = 1e-3;

/// Window areas to fit the playfield into: wide, tall, exact and tiny.
const AREAS: [Vec2; 5] = [
    vec2(1920.0, 1080.0),
    vec2(600.0, 1000.0),
    vec2(800.0, 600.0),
    vec2(333.0, 250.0),
    vec2(1.0, 1.0),
];

fn aspect() -> f32 {
    f32::from(BOARD_W) / f32::from(BOARD_H)
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
    let field = letterbox(Rect::from_min_size(Pos2::ZERO, vec2(1600.0, 600.0)));
    assert_near(field.min, pos2(400.0, 0.0));
    assert_near(field.max, pos2(1200.0, 600.0));
}

#[test]
fn an_empty_area_gives_an_empty_playfield() {
    let field = letterbox(Rect::from_min_size(Pos2::ZERO, Vec2::ZERO));
    assert_eq!(field.size(), Vec2::ZERO);
}

#[test]
fn playfield_corners_map_to_the_letterbox_corners() {
    let available = area(vec2(1000.0, 500.0));
    let transform = to_screen(available);
    let units = playfield_units();
    let field = letterbox(available);
    assert_near(transform.transform_pos(units.min), field.min);
    assert_near(transform.transform_pos(units.max), field.max);
}

/// Runs one headless egui frame showing `game`'s playfield and returns
/// the shapes it painted, after checking that they tessellate.
fn painted(game: &Game, overlay: bool) -> Vec<Shape> {
    let ctx = egui::Context::default();
    let input = egui::RawInput {
        screen_rect: Some(area(vec2(800.0, 600.0))),
        ..egui::RawInput::default()
    };
    let mut output = ctx.run_ui(input, |ui| show(ui, game, overlay));
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
    let polygons = count(&shapes, |s| matches!(s, Shape::Path(p) if p.closed));
    assert_eq!(meshes, 1, "{shapes:?}");
    // The four sides, cut short by the claimed corner, and its two edges.
    assert_eq!(segments, 6, "{shapes:?}");
    assert_eq!(polygons, 1, "the marker: {shapes:?}");
    assert_eq!(count(&shapes, |s| matches!(s, Shape::Text(_))), 0);
}

#[test]
fn the_overlay_adds_its_text() {
    let shapes = painted(&Game::new(Rng::new(SEED)), true);
    assert_eq!(count(&shapes, |s| matches!(s, Shape::Text(_))), 1);
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
