//! Whether the Qix touches the stix or the marker drawing it.

use crate::geometry::{Point, in_triangle, segments_intersect};
use crate::qix::Qix;
use crate::stix::Stix;
use crate::tuning::MARKER_HIT_HALF_SIZE;
use crate::vec2::Vec2;

/// One of `qix` touches `stix` (see [`qix_hits_stix`]) or the marker
/// drawing it at `marker` (see [`qix_hits_marker`]).
pub fn qix_hits_player(qix: &[Qix], stix: &Stix, marker: Point) -> bool {
    qix.iter()
        .any(|qix| qix_hits_stix(qix, stix) || qix_hits_marker(qix, marker))
}

/// The Qix touched one of `stix`'s segments over its last tick: the area
/// its segment swept from [`Qix::previous`] to where it is now meets one,
/// so a Qix crossing the stix within a tick hits it too.
pub fn qix_hits_stix(qix: &Qix, stix: &Stix) -> bool {
    sweep_hits_stix(qix.previous(), (qix.a(), qix.b()), stix)
}

/// The Qix's segment, where it is now, touches one of `stix`'s segments:
/// unlike [`qix_hits_stix`], where it was a tick ago does not count.
pub fn qix_touches_stix(qix: &Qix, stix: &Stix) -> bool {
    let now = (qix.a(), qix.b());
    sweep_hits_stix(now, now, stix)
}

/// The area a segment sweeps moving from `before` to `after` meets one of
/// `stix`'s segments. The area is taken as the convex hull of the two
/// segments' four ends, which holds every segment in between, even one
/// turned past itself; the four triangles of three of the ends cover that
/// hull. A segment that did not move sweeps only itself.
pub fn sweep_hits_stix(before: (Vec2, Vec2), after: (Vec2, Vec2), stix: &Stix) -> bool {
    let ((a0, b0), (a1, b1)) = (before, after);
    let hull = [[a0, b0, b1], [a0, b1, a1], [a0, b0, a1], [b0, b1, a1]];
    stix.segments().any(|(from, to)| {
        let (c, d) = (from.into(), to.into());
        hull.iter()
            .any(|&triangle| segment_meets_triangle(c, d, triangle))
    })
}

/// The segment from `c` to `d` crosses a side of `triangle` or lies inside
/// it.
fn segment_meets_triangle(c: Vec2, d: Vec2, triangle: [Vec2; 3]) -> bool {
    let crosses_side = (0..triangle.len()).any(|i| {
        let (p, q) = (triangle[i], triangle[(i + 1) % triangle.len()]);
        segments_intersect(c, d, p, q)
    });
    crosses_side || in_triangle(c, triangle)
}

/// The Qix's segment touches the square of half side
/// [`MARKER_HIT_HALF_SIZE`] centred on the marker at `marker`: it crosses
/// one of the square's sides or lies inside it.
pub fn qix_hits_marker(qix: &Qix, marker: Point) -> bool {
    let corners = square_corners(marker.into(), MARKER_HIT_HALF_SIZE);
    let crosses_side = (0..corners.len()).any(|i| {
        let (c, d) = (corners[i], corners[(i + 1) % corners.len()]);
        segments_intersect(qix.a(), qix.b(), c, d)
    });
    crosses_side || in_square(qix.a(), marker.into(), MARKER_HIT_HALF_SIZE)
}

/// The corners of the square of half side `half` centred on `centre`, in
/// order around it.
fn square_corners(centre: Vec2, half: f32) -> [Vec2; 4] {
    [
        centre + Vec2::new(-half, -half),
        centre + Vec2::new(half, -half),
        centre + Vec2::new(half, half),
        centre + Vec2::new(-half, half),
    ]
}

/// `point` lies in the square of half side `half` centred on `centre`.
fn in_square(point: Vec2, centre: Vec2, half: f32) -> bool {
    (point.x - centre.x).abs() <= half && (point.y - centre.y).abs() <= half
}
