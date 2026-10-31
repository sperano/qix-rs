//! The stix: the line the marker draws into free space, and the moves
//! that may extend it.

use crate::board::{Board, DrawKind};
use crate::geometry::{Dir, Point};
use crate::marker::Marker;

/// A stix: a polyline of lattice points from where it left the border to
/// the marker, with the button that drew each segment.
///
/// Segments are straight, axis-aligned runs between consecutive points;
/// `kinds[i]` is the kind of the segment from `points[i]` to
/// `points[i + 1]`. A step in the head segment's direction with the same
/// button lengthens that segment instead of starting a new one.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Stix {
    points: Vec<Point>,
    kinds: Vec<DrawKind>,
}

impl Stix {
    /// A stix leaving the border at `start`, with no segment yet.
    pub fn new(start: Point) -> Self {
        Self {
            points: vec![start],
            kinds: Vec::new(),
        }
    }

    /// Where the stix left the border.
    pub fn start(&self) -> Point {
        self.points[0]
    }

    /// The end of the stix, where the marker is.
    pub fn head(&self) -> Point {
        self.points[self.points.len() - 1]
    }

    /// The polyline's points, from the start to the head.
    pub fn points(&self) -> &[Point] {
        &self.points
    }

    /// The draw kind of each segment, from the start to the head.
    pub fn kinds(&self) -> &[DrawKind] {
        &self.kinds
    }

    /// The kind of claim the stix makes: [`DrawKind::Slow`] only when it
    /// has segments and every one was drawn slow, otherwise
    /// [`DrawKind::Fast`].
    pub fn claim_kind(&self) -> DrawKind {
        let all_slow = self.kinds.iter().all(|&kind| kind == DrawKind::Slow);
        if all_slow && !self.kinds.is_empty() {
            DrawKind::Slow
        } else {
            DrawKind::Fast
        }
    }

    /// `point` lies on the stix: on one of its segments, or at its start
    /// when it has none.
    pub fn contains_point(&self, point: Point) -> bool {
        self.start() == point || self.segments().any(|(a, b)| on_segment(a, b, point))
    }

    /// The stix runs from `a` to `b` along one of its segments, in either
    /// direction. For two neighbouring lattice points, this is whether the
    /// stix covers the unit edge between them.
    pub fn contains_segment(&self, a: Point, b: Point) -> bool {
        a != b
            && self
                .segments()
                .any(|(from, to)| on_segment(from, to, a) && on_segment(from, to, b))
    }

    /// Extends the stix from its head to `next`, drawn with `kind`.
    ///
    /// Whether the move is legal is [`legal_step`]'s business; this only
    /// records it.
    ///
    /// # Panics
    ///
    /// If `next` is the head or shares neither its row nor its column.
    pub fn push(&mut self, next: Point, kind: DrawKind) {
        let head = self.head();
        let dir = Dir::between(head, next);
        assert!(
            dir.is_some(),
            "stix step {head:?} -> {next:?} is not straight"
        );
        let continues = match self.points[..] {
            [.., before, _] => Dir::between(before, head) == dir,
            _ => false,
        };
        if continues && self.kinds.last() == Some(&kind) {
            let last = self.points.len() - 1;
            self.points[last] = next;
        } else {
            self.points.push(next);
            self.kinds.push(kind);
        }
    }

    /// The direction of the step from `point` one lattice point along the
    /// stix towards its head; `None` at the head or off the stix. A
    /// corner of the polyline leads along the segment it starts.
    pub fn towards_head(&self, point: Point) -> Option<Dir> {
        let (_, end) = self
            .segments()
            .find(|&(from, to)| point != to && on_segment(from, to, point))?;
        Dir::between(point, end)
    }

    /// The segments, as their two ends, from the start to the head.
    pub fn segments(&self) -> impl Iterator<Item = (Point, Point)> + '_ {
        self.points.windows(2).map(|pair| (pair[0], pair[1]))
    }
}

/// The point one lattice step from the marker towards `dir`, if the stix
/// may be drawn there; the marker is at the stix's head.
///
/// The step must run through free space (both cells beside it free), and
/// its end may not lie on the stix already, except the start reached over
/// an edge the stix does not cover: a loop from a corner of the border
/// back to it. The end is then either a free point inside the free area or
/// a border point, which closes the stix. The rule refuses crossing the
/// stix, retracing it and turning back on the last segment; a step along
/// lattice lines cannot cross a segment without ending on it, so there is
/// no separate crossing check.
pub fn legal_step(marker: &Marker, stix: &Stix, board: &Board, dir: Dir) -> Option<Point> {
    debug_assert_eq!(marker.pos, stix.head(), "the marker is not at the head");
    let next = dir.step(marker.pos)?;
    let loops_to_start = next == stix.start() && !stix.contains_segment(marker.pos, next);
    let free_end = !stix.contains_point(next) || loops_to_start;
    (board.is_free_edge(marker.pos, dir) && free_end).then_some(next)
}

/// The directions the marker may draw in, in the order of [`Dir::ALL`].
/// An empty result means it is caught in a dead end (the "spiral death trap").
pub fn legal_moves(marker: &Marker, stix: &Stix, board: &Board) -> Vec<Dir> {
    Dir::ALL
        .into_iter()
        .filter(|&dir| legal_step(marker, stix, board, dir).is_some())
        .collect()
}

/// `point` lies on the axis-aligned segment from `a` to `b`, ends
/// included.
fn on_segment(a: Point, b: Point, point: Point) -> bool {
    let in_range = |p: u16, q: u16, v: u16| p.min(q) <= v && v <= p.max(q);
    in_range(a.x, b.x, point.x) && in_range(a.y, b.y, point.y)
}
