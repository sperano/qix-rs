use eframe::egui::{self, Color32, Pos2, Rect, Shape, emath::RectTransform, pos2};
use qix_core::tuning::{BOARD_H, BOARD_W, FUSE_DELAY_TICKS, MARKER_STEP_TICKS, QIX_TRAIL_LEN};
use qix_core::{Board, Dir, Game, InputState, Point, Qix, Rng, Sparx, SparxKind};

use crate::palette;
use crate::render::enemies::{
    FUSE_GLOW_RINGS, FUSE_RADIUS, SPARK_FLICKER_TICKS, SPARK_RAYS, fuse_shapes, paint, spark_rays,
    trail_colours,
};

/// Seed of every game the tests play.
const SEED: u64 = 1;

/// Tolerance for comparing screen coordinates, in points.
const EPSILON: f32 = 1e-3;

/// Screen points per playfield unit in [`painted`].
const SCALE: f32 = 5.0;

/// A centre to draw sparks and fuses around.
const CENTER: Pos2 = pos2(40.0, 30.0);

fn colours(len: usize, ticks: u64) -> Vec<Color32> {
    trail_colours(len, ticks).collect()
}

#[test]
fn a_fresh_trail_has_the_first_colour() {
    assert_eq!(colours(1, 0), [palette::QIX_TRAIL[0]]);
}

#[test]
fn each_segment_takes_the_next_colour_oldest_first() {
    assert_eq!(colours(3, 2), palette::QIX_TRAIL[..3]);
}

#[test]
fn the_colours_wrap_around_the_palette() {
    let cycle = palette::QIX_TRAIL.len();
    let len = cycle + 2;
    let expected: Vec<_> = (0..len).map(|i| palette::QIX_TRAIL[i % cycle]).collect();
    assert_eq!(colours(len, len as u64 - 1), expected);
}

#[test]
fn a_segment_keeps_its_colour_while_the_trail_moves_on() {
    for ticks in 0..2 * palette::QIX_TRAIL.len() as u64 {
        let before = colours(QIX_TRAIL_LEN, ticks + 100);
        let after = colours(QIX_TRAIL_LEN, ticks + 101);
        assert_eq!(after[..QIX_TRAIL_LEN - 1], before[1..], "at tick {ticks}");
    }
}

#[test]
fn a_trail_longer_than_the_ticks_still_cycles_backwards() {
    let last = palette::QIX_TRAIL.len() - 1;
    assert_eq!(
        colours(2, 0),
        [palette::QIX_TRAIL[last], palette::QIX_TRAIL[0]]
    );
}

fn half_lengths(rays: [[Pos2; 2]; SPARK_RAYS]) -> Vec<f32> {
    rays.iter().map(|[a, b]| a.distance(*b) / 2.0).collect()
}

#[test]
fn every_ray_of_a_spark_crosses_its_centre() {
    for [a, b] in spark_rays(CENTER, 0) {
        assert!(a.lerp(b, 0.5).distance(CENTER) < EPSILON, "{a:?} {b:?}");
    }
}

#[test]
fn every_other_ray_of_a_spark_is_long() {
    let lengths = half_lengths(spark_rays(CENTER, 0));
    assert!(lengths[0] > lengths[1], "{lengths:?}");
    assert_eq!(lengths[0], lengths[2]);
    assert_eq!(lengths[1], lengths[3]);
}

#[test]
fn a_spark_flickers_by_swapping_its_long_and_short_rays() {
    let still = spark_rays(CENTER, SPARK_FLICKER_TICKS - 1);
    assert_eq!(still, spark_rays(CENTER, 0));
    let mut swapped = half_lengths(spark_rays(CENTER, SPARK_FLICKER_TICKS));
    swapped.rotate_left(1);
    for (swapped, still) in swapped.iter().zip(half_lengths(still)) {
        assert!((swapped - still).abs() < EPSILON, "{swapped} {still}");
    }
}

fn circle(shape: &Shape) -> Option<(Pos2, f32, Color32)> {
    match shape {
        Shape::Circle(circle) => Some((circle.center, circle.radius, circle.fill)),
        _ => None,
    }
}

#[test]
fn the_fuse_is_a_bright_dot_on_top_of_its_glow() {
    let shapes = fuse_shapes(CENTER);
    assert_eq!(shapes.len(), FUSE_GLOW_RINGS + 1);
    let dot = shapes.last().and_then(circle);
    assert_eq!(dot, Some((CENTER, FUSE_RADIUS, palette::FUSE)));
}

#[test]
fn the_glow_is_translucent_rings_shrinking_towards_the_dot() {
    let rings: Vec<_> = fuse_shapes(CENTER).iter().filter_map(circle).collect();
    let (dot, glow) = rings.split_last().expect("no dot");
    for pair in glow.windows(2) {
        assert!(pair[0].1 > pair[1].1, "{rings:?}");
    }
    for &(center, radius, fill) in glow {
        assert_eq!(center, CENTER);
        assert!(radius > dot.1, "{rings:?}");
        assert!(fill.a() < u8::MAX, "{fill:?}");
    }
}

/// Runs one headless egui frame painting `game`'s enemies at [`SCALE`]
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
    let mut output = ctx.run_ui(input, |ui| paint(ui.painter(), to_screen, game));
    // No renderer uploads the font atlas here; dropping it unapplied panics.
    output.textures_delta.clear();
    output
        .shapes
        .into_iter()
        .map(|clipped| clipped.shape)
        .collect()
}

/// The shapes painted as line segments in `colour`.
fn segments_in(shapes: &[Shape], colour: Color32) -> usize {
    let in_colour =
        |s: &&Shape| matches!(s, Shape::LineSegment { stroke, .. } if stroke.color == colour);
    shapes.iter().filter(in_colour).count()
}

/// A Qix standing still across the middle of the board.
fn still_qix() -> Qix {
    let mid = f32::from(BOARD_H) / 2.0;
    Qix::new(
        qix_core::Vec2::new(70.0, mid),
        qix_core::Vec2::new(90.0, mid),
        qix_core::Vec2::default(),
        0.0,
    )
}

fn game_with(sparx: Vec<Sparx>) -> Game {
    let board = Board::new(BOARD_W, BOARD_H);
    Game::with_board(board, still_qix(), sparx, Rng::new(SEED))
}

#[test]
fn the_qix_is_painted_as_its_whole_trail() {
    let mut game = game_with(Vec::new());
    let ticks = palette::QIX_TRAIL.len();
    for _ in 0..ticks {
        game.tick(&InputState::default());
    }
    let shapes = painted(&game);
    assert_eq!(game.qix().trail().len(), ticks + 1);
    for colour in palette::QIX_TRAIL {
        let expected = if colour == palette::QIX_TRAIL[0] {
            2
        } else {
            1
        };
        assert_eq!(segments_in(&shapes, colour), expected, "{colour:?}");
    }
}

#[test]
fn each_sparx_is_a_spark_in_the_colour_of_its_kind() {
    let at = |x, kind| Sparx {
        pos: Point::new(x, 0),
        dir: Dir::Right,
        kind,
    };
    let game = game_with(vec![
        at(10, SparxKind::Normal),
        at(50, SparxKind::Normal),
        at(100, SparxKind::Super),
    ]);
    let shapes = painted(&game);
    assert_eq!(segments_in(&shapes, palette::SPARX), 2 * SPARK_RAYS);
    assert_eq!(segments_in(&shapes, palette::SUPER_SPARX), SPARK_RAYS);
}

#[test]
fn no_fuse_is_painted_while_it_is_out() {
    let shapes = painted(&game_with(Vec::new()));
    assert_eq!(shapes.iter().filter_map(circle).count(), 0);
}

#[test]
fn a_lit_fuse_is_painted_at_the_start_of_the_stix() {
    let mut game = game_with(Vec::new());
    let drawing_up = InputState {
        dir: Some(Dir::Up),
        fast: true,
        ..InputState::default()
    };
    for _ in 0..MARKER_STEP_TICKS {
        game.tick(&drawing_up);
    }
    let standing = InputState {
        fast: true,
        ..InputState::default()
    };
    for _ in 0..FUSE_DELAY_TICKS {
        game.tick(&standing);
    }
    assert!(game.fuse().is_some(), "the fuse is not lit");
    let start = game.stix().expect("no stix").start();
    let at = pos2(f32::from(start.x), f32::from(start.y)) * SCALE;
    let shapes = painted(&game);
    let dots: Vec<_> = shapes.iter().filter_map(circle).collect();
    assert_eq!(dots.len(), FUSE_GLOW_RINGS + 1, "{dots:?}");
    assert!(
        dots.iter()
            .all(|&(center, ..)| center.distance(at) < EPSILON)
    );
}
