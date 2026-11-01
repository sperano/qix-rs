use std::ops::Range;

use eframe::egui::{self, Color32, Pos2, Rect, Shape, TextureId, emath::RectTransform, pos2};
use eframe::epaint::ColorMode;
use qix_core::tuning::{BOARD_H, BOARD_W};
use qix_core::{Board, Cell, Dir, DrawKind, Game, InputState, Point, Rng, Stix};

use crate::palette;
use crate::render::board::{
    CellRun, STIX_FLICKER_TICKS, border_segments, claimed_runs, marker_inner, paint, square_capped,
    stix_colour, stix_runs,
};
use crate::render::sprite;

/// Seed of every game the tests play.
const SEED: u64 = 1;

/// Tolerance for comparing screen coordinates, in points.
const EPSILON: f32 = 1e-3;

/// Dimensions of the small board the tests use.
const W: u16 = 6;
const H: u16 = 4;

const FAST: Cell = Cell::Claimed(DrawKind::Fast);
const SLOW: Cell = Cell::Claimed(DrawKind::Slow);

fn run(row: u16, cols: Range<u16>, kind: DrawKind) -> CellRun {
    CellRun { row, cols, kind }
}

fn segment(from: (u16, u16), to: (u16, u16)) -> (Point, Point) {
    (Point::new(from.0, from.1), Point::new(to.0, to.1))
}

/// A `W x H` board with the cells of each `(rows, cols, cell)` block set to
/// `cell`.
fn board_with(cells: &[(Range<u16>, Range<u16>, Cell)]) -> Board {
    let mut board = Board::new(W, H);
    for (rows, cols, cell) in cells {
        for y in rows.clone() {
            for x in cols.clone() {
                board.set_cell(x, y, *cell);
            }
        }
    }
    board
}

#[test]
fn a_fresh_board_has_no_claimed_runs() {
    assert!(claimed_runs(&Board::new(W, H)).is_empty());
}

#[test]
fn neighbouring_cells_of_one_kind_merge_into_one_run() {
    let board = board_with(&[(1..2, 1..4, FAST)]);
    assert_eq!(claimed_runs(&board), [run(1, 1..4, DrawKind::Fast)]);
}

#[test]
fn a_change_of_kind_or_a_free_cell_ends_a_run() {
    let board = board_with(&[(0..1, 0..2, FAST), (0..1, 2..3, SLOW), (0..1, 4..6, SLOW)]);
    assert_eq!(
        claimed_runs(&board),
        [
            run(0, 0..2, DrawKind::Fast),
            run(0, 2..3, DrawKind::Slow),
            run(0, 4..6, DrawKind::Slow),
        ]
    );
}

#[test]
fn runs_never_span_two_rows() {
    let board = board_with(&[(2..4, 0..W, SLOW)]);
    assert_eq!(
        claimed_runs(&board),
        [run(2, 0..W, DrawKind::Slow), run(3, 0..W, DrawKind::Slow)]
    );
}

#[test]
fn a_run_covers_its_cells_in_playfield_units() {
    let rect = run(2, 1..4, DrawKind::Fast).rect();
    assert_eq!(rect, Rect::from_min_max(pos2(1.0, 2.0), pos2(4.0, 3.0)));
}

#[test]
fn the_border_of_a_fresh_board_is_its_four_sides() {
    assert_eq!(
        border_segments(&Board::new(W, H)),
        [
            segment((0, 0), (W, 0)),
            segment((0, H), (W, H)),
            segment((0, 0), (0, H)),
            segment((W, 0), (W, H)),
        ]
    );
}

#[test]
fn a_claimed_corner_moves_the_border_around_it() {
    let board = board_with(&[(0..2, 0..3, FAST)]);
    assert_eq!(
        border_segments(&board),
        [
            segment((3, 0), (W, 0)),
            segment((0, 2), (3, 2)),
            segment((0, H), (W, H)),
            segment((0, 2), (0, H)),
            segment((3, 0), (3, 2)),
            segment((W, 0), (W, H)),
        ]
    );
}

#[test]
fn claimed_cells_of_both_kinds_have_no_border_between_them() {
    let board = board_with(&[(0..H, 0..3, FAST), (0..H, 3..W, SLOW)]);
    assert!(border_segments(&board).is_empty());
}

#[test]
fn a_square_cap_extends_a_line_by_half_its_width_at_both_ends() {
    let horizontal = square_capped([pos2(10.0, 5.0), pos2(30.0, 5.0)], 2.0);
    assert_eq!(horizontal, [pos2(9.0, 5.0), pos2(31.0, 5.0)]);
    let vertical = square_capped([pos2(4.0, 20.0), pos2(4.0, 8.0)], 4.0);
    assert_eq!(vertical, [pos2(4.0, 22.0), pos2(4.0, 6.0)]);
}

#[test]
fn a_stix_without_segments_has_no_runs() {
    assert!(stix_runs(&Stix::new(Point::new(2, H))).is_empty());
}

#[test]
fn stix_segments_of_one_kind_form_one_polyline() {
    let mut stix = Stix::new(Point::new(2, H));
    stix.push(Point::new(2, 2), DrawKind::Fast);
    stix.push(Point::new(4, 2), DrawKind::Fast);
    let expected = [Point::new(2, H), Point::new(2, 2), Point::new(4, 2)];
    assert_eq!(stix_runs(&stix), [(DrawKind::Fast, &expected[..])]);
}

#[test]
fn a_change_of_kind_starts_a_polyline_at_the_last_point() {
    let mut stix = Stix::new(Point::new(2, H));
    stix.push(Point::new(2, 2), DrawKind::Slow);
    stix.push(Point::new(4, 2), DrawKind::Fast);
    stix.push(Point::new(4, 1), DrawKind::Fast);
    stix.push(Point::new(5, 1), DrawKind::Slow);
    let slow_start = [Point::new(2, H), Point::new(2, 2)];
    let fast = [Point::new(2, 2), Point::new(4, 2), Point::new(4, 1)];
    let slow_end = [Point::new(4, 1), Point::new(5, 1)];
    assert_eq!(
        stix_runs(&stix),
        [
            (DrawKind::Slow, &slow_start[..]),
            (DrawKind::Fast, &fast[..]),
            (DrawKind::Slow, &slow_end[..]),
        ]
    );
}

#[test]
fn a_stix_starts_in_the_colour_of_its_kind() {
    assert_eq!(stix_colour(DrawKind::Fast, 0), palette::FAST_STIX);
    assert_eq!(stix_colour(DrawKind::Slow, 0), palette::SLOW_STIX);
}

#[test]
fn a_stix_flickers_to_the_bright_colour_and_back_every_flicker_period() {
    let kind = DrawKind::Fast;
    let last_still = STIX_FLICKER_TICKS - 1;
    assert_eq!(stix_colour(kind, last_still), palette::FAST_STIX);
    assert_eq!(stix_colour(kind, STIX_FLICKER_TICKS), palette::STIX_FLICKER);
    assert_eq!(
        stix_colour(kind, 2 * STIX_FLICKER_TICKS - 1),
        palette::STIX_FLICKER
    );
    assert_eq!(
        stix_colour(kind, 2 * STIX_FLICKER_TICKS),
        palette::FAST_STIX
    );
}

#[test]
fn the_flicker_period_is_the_same_for_both_kinds() {
    for ticks in 0..4 * STIX_FLICKER_TICKS {
        let flickering = |kind| stix_colour(kind, ticks) == palette::STIX_FLICKER;
        assert_eq!(
            flickering(DrawKind::Fast),
            flickering(DrawKind::Slow),
            "{ticks}"
        );
    }
}

#[test]
fn the_marker_is_plain_until_the_stix_has_a_segment() {
    assert_eq!(marker_inner(None), palette::MARKER);
    assert_eq!(
        marker_inner(Some(&Stix::new(Point::new(2, H)))),
        palette::MARKER
    );
}

#[test]
fn the_marker_takes_the_colour_of_the_kind_of_the_last_segment() {
    let mut stix = Stix::new(Point::new(2, H));
    stix.push(Point::new(2, 2), DrawKind::Fast);
    assert_eq!(marker_inner(Some(&stix)), palette::FAST_STIX);
    stix.push(Point::new(4, 2), DrawKind::Slow);
    assert_eq!(marker_inner(Some(&stix)), palette::SLOW_STIX);
}

/// Screen points per playfield unit in [`painted`].
const SCALE: f32 = 5.0;

/// The texture of the claimed cells [`painted`] hands the board.
const CLAIMED_TEXTURE: TextureId = TextureId::User(1);

/// Runs one headless egui frame painting `game`'s board at [`SCALE`]
/// points per unit and returns the shapes.
fn painted(game: &Game) -> Vec<Shape> {
    let units = Rect::from_min_max(Pos2::ZERO, pos2(f32::from(BOARD_W), f32::from(BOARD_H)));
    let to_screen =
        RectTransform::from_to(units, Rect::from_min_size(Pos2::ZERO, units.size() * SCALE));
    let ctx = egui::Context::default();
    let input = egui::RawInput {
        screen_rect: Some(*to_screen.to()),
        ..egui::RawInput::default()
    };
    let mut output = ctx.run_ui(input, |ui| {
        paint(ui.painter(), to_screen, game, CLAIMED_TEXTURE, None)
    });
    // No renderer uploads the font atlas here; dropping it unapplied panics.
    output.textures_delta.clear();
    output
        .shapes
        .into_iter()
        .map(|clipped| clipped.shape)
        .collect()
}

#[test]
fn the_claimed_cells_are_painted_first_as_their_texture_over_the_board() {
    let shapes = painted(&Game::new(Rng::new(SEED)));
    let Some(Shape::Mesh(mesh)) = shapes.first() else {
        panic!("the claimed cells are not painted first: {shapes:?}");
    };
    assert_eq!(mesh.texture_id, CLAIMED_TEXTURE);
    let board = pos2(f32::from(BOARD_W) * SCALE, f32::from(BOARD_H) * SCALE);
    assert_eq!(mesh.calc_bounds(), Rect::from_min_max(Pos2::ZERO, board));
}

#[test]
fn the_marker_is_painted_last_as_a_sprite_centred_on_its_position() {
    let game = Game::new(Rng::new(SEED));
    let shapes = painted(&game);
    let Some(Shape::Mesh(mesh)) = shapes.last() else {
        panic!("{shapes:?}");
    };
    let at = pos2(
        f32::from(game.marker().pos.x),
        f32::from(game.marker().pos.y),
    ) * SCALE;
    let points: Vec<_> = mesh.vertices.iter().map(|v| v.pos).collect();
    let centre = Rect::from_points(&points).center();
    assert!(centre.distance(at) < EPSILON, "{centre:?} {at:?}");
    assert_eq!(**mesh, sprite::marker(at, sprite::PIXEL, palette::MARKER));
}

/// A game whose marker has drawn a fast stix of one segment, at rest on
/// its first stix point.
fn drawing_game(extra_ticks: u64) -> Game {
    let mut game = Game::new(Rng::new(SEED));
    let up = InputState {
        dir: Some(Dir::Up),
        fast: true,
        ..InputState::default()
    };
    while game.stix().is_none() {
        game.tick(&up);
    }
    for _ in 0..extra_ticks {
        game.tick(&InputState::default());
    }
    game
}

/// The colours of the stix polylines among `shapes`.
fn stix_colours(shapes: &[Shape]) -> Vec<Color32> {
    shapes
        .iter()
        .filter_map(|shape| match shape {
            Shape::Path(path) if !path.closed => match path.stroke.color {
                ColorMode::Solid(colour) => Some(colour),
                ColorMode::UV(_) => None,
            },
            _ => None,
        })
        .collect()
}

#[test]
fn the_painted_stix_flickers_with_the_game_ticks() {
    let still = drawing_game(0);
    let expected = |game: &Game| vec![stix_colour(DrawKind::Fast, game.ticks())];
    assert_eq!(stix_colours(&painted(&still)), expected(&still));
    let flickered = drawing_game(STIX_FLICKER_TICKS);
    assert_ne!(
        stix_colour(DrawKind::Fast, still.ticks()),
        stix_colour(DrawKind::Fast, flickered.ticks())
    );
    assert_eq!(stix_colours(&painted(&flickered)), expected(&flickered));
}

#[test]
fn the_marker_takes_the_stix_colour_while_drawing() {
    let game = drawing_game(0);
    let shapes = painted(&game);
    let Some(Shape::Mesh(mesh)) = shapes.last() else {
        panic!("{shapes:?}");
    };
    let colours: Vec<_> = mesh.vertices.iter().map(|v| v.color).collect();
    assert!(colours.contains(&palette::FAST_STIX));
    assert!(!colours.contains(&palette::MARKER));
}
