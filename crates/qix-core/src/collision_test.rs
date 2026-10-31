use crate::collision::{qix_hits_marker, qix_hits_stix};
use crate::tuning::MARKER_HIT_HALF_SIZE;
use crate::{DrawKind, Point, Qix, Stix, Vec2};

/// How far outside the marker's square the near misses pass.
const MISS_GAP: f32 = 0.1;

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
