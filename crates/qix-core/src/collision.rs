//! Whether the Qix touches the stix or the marker drawing it.

use crate::geometry::{Point, segments_intersect};
use crate::qix::Qix;
use crate::stix::Stix;
use crate::tuning::MARKER_HIT_HALF_SIZE;
use crate::vec2::Vec2;

/// The Qix's segment touches one of `stix`'s segments.
pub fn qix_hits_stix(qix: &Qix, stix: &Stix) -> bool {
    stix.segments()
        .any(|(from, to)| segments_intersect(qix.a(), qix.b(), from.into(), to.into()))
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
