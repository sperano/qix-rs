use std::f32::consts::FRAC_1_SQRT_2;

use eframe::egui::{self, Color32, Pos2, Rect, Shape, Vec2, emath::RectTransform, pos2, vec2};
use qix_core::tuning::{
    BOARD_H, BOARD_W, FUSE_DELAY_TICKS, FUSE_STEP_TICKS, MARKER_STEP_TICKS, QIX_TRAIL_LEN,
    TWO_QIX_FROM_LEVEL,
};
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
fn the_rays_of_a_spark_are_horizontal_diagonal_vertical_and_antidiagonal() {
    let expected = [
        vec2(1.0, 0.0),
        vec2(FRAC_1_SQRT_2, FRAC_1_SQRT_2),
        vec2(0.0, 1.0),
        vec2(-FRAC_1_SQRT_2, FRAC_1_SQRT_2),
    ];
    for ([a, b], expected) in spark_rays(CENTER, 0).into_iter().zip(expected) {
        let dir = (b - a).normalized();
        assert!((dir - expected).length() < EPSILON, "{dir:?} {expected:?}");
    }
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

/// The ends and colour of each shape painted as a line segment, in
/// painting order.
fn lines(shapes: &[Shape]) -> Vec<([Pos2; 2], Color32)> {
    shapes
        .iter()
        .filter_map(|shape| match shape {
            Shape::LineSegment { points, stroke } => Some((*points, stroke.color)),
            _ => None,
        })
        .collect()
}

/// `(x, y)` in playfield units, on the screen of [`painted`].
fn on_screen(x: f32, y: f32) -> Pos2 {
    pos2(x, y) * SCALE
}

/// A Qix across the middle of the board, at rest until its first tick
/// kicks it to its slowest speed.
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
    Game::with_board(board, vec![still_qix()], sparx, Rng::new(SEED))
}

/// Ticks `game` `ticks` times with nothing held.
fn idle(game: &mut Game, ticks: usize) {
    for _ in 0..ticks {
        game.tick(&InputState::default());
    }
}

#[test]
fn the_qix_is_painted_as_its_whole_trail() {
    let mut game = game_with(Vec::new());
    let ticks = palette::QIX_TRAIL.len();
    idle(&mut game, ticks);
    let shapes = painted(&game);
    assert_eq!(game.qix()[0].trail().len(), ticks + 1);
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
fn each_trail_segment_is_painted_on_its_ends_oldest_first() {
    let mut game = game_with(Vec::new());
    idle(&mut game, QIX_TRAIL_LEN + 1);
    let trail = game.qix()[0].trail();
    assert_ne!(trail.front(), trail.back(), "the Qix did not move");
    let cycle = palette::QIX_TRAIL.len() as u64;
    let painted = lines(&painted(&game));
    assert_eq!(painted.len(), trail.len());
    for (i, (&(a, b), (ends, colour))) in trail.iter().zip(painted).enumerate() {
        let age = (trail.len() - 1 - i) as u64;
        let expected = [on_screen(a.x, a.y), on_screen(b.x, b.y)];
        for (end, expected) in ends.into_iter().zip(expected) {
            assert!(end.distance(expected) < EPSILON, "segment {i}: {end:?}");
        }
        let tick = (game.ticks() - age) % cycle;
        assert_eq!(colour, palette::QIX_TRAIL[tick as usize], "segment {i}");
    }
}

#[test]
fn every_qix_of_a_two_qix_level_is_painted() {
    let mut game = Game::new(Rng::new(SEED));
    game.start_level(TWO_QIX_FROM_LEVEL);
    idle(&mut game, 1);
    assert_eq!(game.qix().len(), 2);
    let painted: Vec<_> = lines(&painted(&game))
        .into_iter()
        .filter(|(_, colour)| palette::QIX_TRAIL.contains(colour))
        .collect();
    let trails: usize = game.qix().iter().map(|qix| qix.trail().len()).sum();
    assert_eq!(painted.len(), trails);
    for qix in game.qix() {
        let (a, b) = (qix.a(), qix.b());
        let ends = [on_screen(a.x, a.y), on_screen(b.x, b.y)];
        let found = painted.iter().any(|(painted, _)| {
            painted
                .iter()
                .zip(ends)
                .all(|(end, expected)| end.distance(expected) < EPSILON)
        });
        assert!(found, "Qix at {a:?}-{b:?} not painted");
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
fn each_spark_is_centred_on_its_sparx() {
    let xs = [10, 50, 100];
    let sparx = xs.map(|x| Sparx {
        pos: Point::new(x, 0),
        dir: Dir::Right,
        kind: SparxKind::Normal,
    });
    let shapes = painted(&game_with(sparx.to_vec()));
    let rays: Vec<_> = lines(&shapes)
        .into_iter()
        .filter(|&(_, colour)| colour == palette::SPARX)
        .map(|(ends, _)| ends)
        .collect();
    assert_eq!(rays.len(), xs.len() * SPARK_RAYS);
    for (spark, x) in rays.chunks(SPARK_RAYS).zip(xs) {
        let at = on_screen(f32::from(x), 0.0);
        for [a, b] in spark {
            assert!(a.lerp(*b, 0.5).distance(at) < EPSILON, "{a:?} {b:?}");
        }
    }
}

fn drawing(dir: Dir) -> InputState {
    InputState {
        dir: Some(dir),
        fast: true,
        ..InputState::default()
    }
}

/// Ticks `game` long enough for `steps` fast stix steps towards `dir`.
fn draw(game: &mut Game, dir: Dir, steps: u32) {
    for _ in 0..steps * MARKER_STEP_TICKS {
        game.tick(&drawing(dir));
    }
}

/// Ticks `game` `ticks` times with the marker standing on its stix.
fn stand(game: &mut Game, ticks: u32) {
    let standing = InputState {
        fast: true,
        ..InputState::default()
    };
    for _ in 0..ticks {
        game.tick(&standing);
    }
}

/// The centres of the circles in `shapes`.
fn circle_centres(shapes: &[Shape]) -> Vec<Pos2> {
    shapes
        .iter()
        .filter_map(circle)
        .map(|(center, ..)| center)
        .collect()
}

/// The fuse's shapes are all centred on `point`.
fn assert_fuse_at(shapes: &[Shape], point: Point) {
    let at = on_screen(f32::from(point.x), f32::from(point.y));
    let centres = circle_centres(shapes);
    assert_eq!(centres.len(), FUSE_GLOW_RINGS + 1, "{centres:?}");
    let offsets: Vec<Vec2> = centres.iter().map(|&center| center - at).collect();
    assert!(
        offsets.iter().all(|offset| offset.length() < EPSILON),
        "{offsets:?}"
    );
}

#[test]
fn no_fuse_is_painted_while_it_is_out() {
    let shapes = painted(&game_with(Vec::new()));
    assert_eq!(shapes.iter().filter_map(circle).count(), 0);
}

#[test]
fn no_fuse_is_painted_on_a_stix_before_it_is_lit() {
    let mut game = game_with(Vec::new());
    draw(&mut game, Dir::Up, 1);
    stand(&mut game, FUSE_DELAY_TICKS - 1);
    assert!(game.stix().is_some(), "no stix");
    assert_eq!(game.fuse(), None);
    assert_eq!(circle_centres(&painted(&game)), []);
}

#[test]
fn a_lit_fuse_is_painted_at_the_start_of_the_stix() {
    let mut game = game_with(Vec::new());
    draw(&mut game, Dir::Up, 1);
    stand(&mut game, FUSE_DELAY_TICKS);
    assert!(game.fuse().is_some(), "the fuse is not lit");
    let start = game.stix().expect("no stix").start();
    assert_fuse_at(&painted(&game), start);
}

#[test]
fn the_fuse_is_painted_on_the_stix_point_it_has_burnt_to() {
    let mut game = game_with(Vec::new());
    draw(&mut game, Dir::Up, 2);
    draw(&mut game, Dir::Right, 2);
    draw(&mut game, Dir::Up, 1);
    stand(&mut game, FUSE_DELAY_TICKS + FUSE_STEP_TICKS);
    let pos = game.fuse().expect("the fuse is not lit").pos;
    let points = game.stix().expect("no stix").points();
    assert!(
        0 < pos && pos < points.len() - 1,
        "fuse at {pos} of {points:?}"
    );
    assert_fuse_at(&painted(&game), points[pos]);
}
