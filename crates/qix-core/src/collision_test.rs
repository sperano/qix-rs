use crate::collision::{
    qix_hits_marker, qix_hits_player, qix_hits_stix, qix_touches_stix, sweep_hits_stix,
};
use crate::tuning::{MARKER_HIT_HALF_SIZE, QIX_SPEED};
use crate::{Board, DrawKind, Point, Qix, Rng, Stix, Vec2};

/// How far outside the marker's square the near misses pass.
const MISS_GAP: f32 = 0.1;

/// The top speed at the end of the difficulty ramp, in playfield units
/// per tick: one tick of it moves a Qix a whole cell.
const TOP_SPEED: f32 = 1.0;

/// Where the vertical Qix of the sweep tests runs: between the rows of
/// [`up_then_right`]'s vertical leg, clear of its horizontal one.
const SWEEP_TOP: f32 = 6.0;
const SWEEP_BOTTOM: f32 = 9.0;

/// Where [`a_qix_crossing_the_stix_within_a_tick_hits_it`] starts its
/// Qix: closer to [`up_then_right`]'s vertical leg, at `x = 2`, than the
/// slowest Qix moves in a tick.
const CROSSING_START_X: f32 = 1.75;

/// The free board that Qix tick on.
const SWEEP_BOARD: u16 = 12;

/// Seed of the Qix that tick.
const SEED: u64 = 1195;

/// The vertical segment at `x` between [`SWEEP_TOP`] and [`SWEEP_BOTTOM`].
fn vertical_at(x: f32) -> (Vec2, Vec2) {
    (Vec2::new(x, SWEEP_TOP), Vec2::new(x, SWEEP_BOTTOM))
}

/// A still Qix on the segment from `a` to `b`.
fn qix(a: (f32, f32), b: (f32, f32)) -> Qix {
    let (a, b) = (Vec2::new(a.0, a.1), Vec2::new(b.0, b.1));
    Qix::new(a, b, Vec2::default(), 0.0)
}

/// A stix up from `(2, 10)` to `(2, 5)`, then right to `(8, 5)`.
fn up_then_right() -> Stix {
    let mut stix = Stix::new(Point::new(2, 10));
    stix.push(Point::new(2, 5), DrawKind::Fast);
    stix.push(Point::new(8, 5), DrawKind::Fast);
    stix
}

#[test]
fn a_qix_crossing_a_horizontal_stix_hits_it() {
    assert!(qix_hits_stix(
        &qix((5.0, 3.0), (5.0, 7.0)),
        &up_then_right()
    ));
}

#[test]
fn a_qix_crossing_a_vertical_stix_hits_it() {
    assert!(qix_hits_stix(
        &qix((1.0, 8.0), (3.5, 7.5)),
        &up_then_right()
    ));
}

#[test]
fn a_qix_ending_on_the_stix_hits_it() {
    assert!(qix_hits_stix(
        &qix((5.0, 2.0), (5.0, 5.0)),
        &up_then_right()
    ));
}

#[test]
fn a_qix_passing_beside_the_stix_misses_it() {
    let stix = up_then_right();
    assert!(!qix_hits_stix(&qix((3.0, 4.5), (7.0, 4.5)), &stix));
    assert!(!qix_hits_stix(&qix((3.0, 6.0), (3.0, 9.0)), &stix));
    assert!(!qix_hits_stix(&qix((9.0, 3.0), (9.0, 7.0)), &stix));
}

#[test]
fn a_qix_through_the_marker_square_hits_the_marker() {
    let marker = Point::new(4, 4);
    assert!(qix_hits_marker(&qix((3.0, 4.2), (5.0, 4.2)), marker));
    assert!(qix_hits_marker(&qix((4.4, 2.0), (4.4, 6.0)), marker));
}

#[test]
fn a_qix_inside_the_marker_square_hits_the_marker() {
    assert!(qix_hits_marker(
        &qix((3.9, 4.0), (4.1, 4.0)),
        Point::new(4, 4)
    ));
}

#[test]
fn a_qix_just_outside_the_marker_square_misses_the_marker() {
    let marker = Point::new(4, 4);
    let near = 4.0 + MARKER_HIT_HALF_SIZE + MISS_GAP;
    let far = 4.0 - MARKER_HIT_HALF_SIZE - MISS_GAP;
    assert!(!qix_hits_marker(&qix((near, 2.0), (near, 6.0)), marker));
    assert!(!qix_hits_marker(&qix((2.0, far), (6.0, far)), marker));
}

#[test]
fn a_segment_crossing_the_stix_within_a_tick_hits_it() {
    let stix = up_then_right();
    let (before, after) = (
        vertical_at(2.0 - TOP_SPEED / 2.0),
        vertical_at(2.0 + TOP_SPEED / 2.0),
    );
    assert!(
        !sweep_hits_stix(before, before, &stix),
        "starts on the stix"
    );
    assert!(!sweep_hits_stix(after, after, &stix), "ends on the stix");

    assert!(sweep_hits_stix(before, after, &stix));
    assert!(sweep_hits_stix(after, before, &stix));
}

#[test]
fn a_stix_inside_the_swept_area_is_hit() {
    let mut stix = Stix::new(Point::new(2, 7));
    stix.push(Point::new(2, 8), DrawKind::Fast);

    assert!(sweep_hits_stix(vertical_at(1.5), vertical_at(2.5), &stix));
}

#[test]
fn a_segment_turning_past_itself_across_the_stix_hits_it() {
    let mut stix = Stix::new(Point::new(3, 7));
    stix.push(Point::new(3, 8), DrawKind::Fast);
    let before = (Vec2::new(0.0, 9.5), Vec2::new(4.0, 5.5));
    let after = (Vec2::new(0.0, 5.5), Vec2::new(4.0, 9.5));
    assert!(
        !sweep_hits_stix(before, before, &stix),
        "starts on the stix"
    );
    assert!(!sweep_hits_stix(after, after, &stix), "ends on the stix");

    assert!(sweep_hits_stix(before, after, &stix));
}

#[test]
fn a_segment_moving_beside_the_stix_misses_it() {
    let stix = up_then_right();

    assert!(!sweep_hits_stix(vertical_at(0.5), vertical_at(1.5), &stix));
    assert!(!sweep_hits_stix(vertical_at(3.0), vertical_at(4.0), &stix));
}

#[test]
fn a_qix_crossing_the_stix_within_a_tick_hits_it() {
    let (a, b) = vertical_at(CROSSING_START_X);
    let mut qix = Qix::new(a, b, Vec2::new(*QIX_SPEED.end(), 0.0), 0.0);
    let stix = up_then_right();
    assert!(!qix_hits_stix(&qix, &stix), "starts on the stix");

    qix.tick(&Board::new(SWEEP_BOARD, SWEEP_BOARD), &mut Rng::new(SEED));

    assert!(qix.a().x > 2.0 && qix.b().x > 2.0, "{qix:?} did not cross");
    assert!(qix_hits_stix(&qix, &stix));
}

#[test]
fn a_qix_that_crossed_the_stix_a_tick_ago_does_not_touch_it() {
    let (a, b) = vertical_at(CROSSING_START_X);
    let mut qix = Qix::new(a, b, Vec2::new(*QIX_SPEED.end(), 0.0), 0.0);
    let stix = up_then_right();
    assert!(!qix_touches_stix(&qix, &stix), "starts on the stix");

    qix.tick(&Board::new(SWEEP_BOARD, SWEEP_BOARD), &mut Rng::new(SEED));

    assert!(qix_hits_stix(&qix, &stix), "{qix:?} did not cross");
    assert!(!qix_touches_stix(&qix, &stix));
}

#[test]
fn a_qix_on_the_stix_touches_it() {
    let stix = up_then_right();
    assert!(qix_touches_stix(&qix((5.0, 3.0), (5.0, 7.0)), &stix));
    assert!(qix_touches_stix(&qix((5.0, 2.0), (5.0, 5.0)), &stix));
    assert!(!qix_touches_stix(&qix((3.0, 6.0), (3.0, 9.0)), &stix));
}

#[test]
fn any_of_the_qix_hits_the_player() {
    let stix = up_then_right();
    let marker = Point::new(8, 5);
    let far = qix((10.0, 10.0), (11.0, 10.0));
    let on_stix = qix((5.0, 3.0), (5.0, 7.0));
    let on_marker = qix((8.2, 3.0), (8.2, 4.8));
    assert!(!qix_hits_stix(&on_marker, &stix), "touches the stix");

    assert!(!qix_hits_player(std::slice::from_ref(&far), &stix, marker));
    for hit in [on_stix, on_marker] {
        let (first, second) = ([far.clone(), hit.clone()], [hit, far.clone()]);
        assert!(qix_hits_player(&first, &stix, marker));
        assert!(qix_hits_player(&second, &stix, marker));
    }
}
