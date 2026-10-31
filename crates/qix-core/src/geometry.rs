//! Geometry of the playfield.

use std::cmp::Ordering;

use crate::vec2::Vec2;

/// A corner of a cell: a point of the `(W+1) x (H+1)` lattice the marker,
/// the stix and the Sparx move on.
///
/// Point `(x, y)` is the top-left corner of cell `(x, y)`; `y` grows
/// downwards.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Point {
    pub x: u16,
    pub y: u16,
}

impl Point {
    /// The lattice point at `(x, y)`.
    pub const fn new(x: u16, y: u16) -> Self {
        Self { x, y }
    }
}

/// One of the four directions the marker moves in.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Dir {
    Up,
    Down,
    Left,
    Right,
}

impl Dir {
    /// Every direction, in declaration order.
    pub const ALL: [Self; 4] = [Self::Up, Self::Down, Self::Left, Self::Right];

    /// The direction pointing the other way.
    pub const fn opposite(self) -> Self {
        match self {
            Self::Up => Self::Down,
            Self::Down => Self::Up,
            Self::Left => Self::Right,
            Self::Right => Self::Left,
        }
    }

    /// The lattice point one step from `point` in this direction, or
    /// `None` when the step leaves the coordinate range.
    ///
    /// The step is not bounded by a board: checking that the result lies
    /// on a given board's lattice is the caller's job.
    pub fn step(self, point: Point) -> Option<Point> {
        let Point { x, y } = point;
        match self {
            Self::Up => y.checked_sub(1).map(|y| Point::new(x, y)),
            Self::Down => y.checked_add(1).map(|y| Point::new(x, y)),
            Self::Left => x.checked_sub(1).map(|x| Point::new(x, y)),
            Self::Right => x.checked_add(1).map(|x| Point::new(x, y)),
        }
    }

    /// The direction from `from` straight to `to`, or `None` when the two
    /// are the same point or share neither a row nor a column.
    pub fn between(from: Point, to: Point) -> Option<Self> {
        match (from.x.cmp(&to.x), from.y.cmp(&to.y)) {
            (Ordering::Equal, Ordering::Greater) => Some(Self::Up),
            (Ordering::Equal, Ordering::Less) => Some(Self::Down),
            (Ordering::Greater, Ordering::Equal) => Some(Self::Left),
            (Ordering::Less, Ordering::Equal) => Some(Self::Right),
            _ => None,
        }
    }
}

/// The closed segments from `a` to `b` and from `c` to `d` share a point:
/// they cross, or one touches the other with an end, or they overlap on a
/// line.
pub fn segments_intersect(a: Vec2, b: Vec2, c: Vec2, d: Vec2) -> bool {
    let (side_a, side_b) = (orientation(c, d, a), orientation(c, d, b));
    let (side_c, side_d) = (orientation(a, b, c), orientation(a, b, d));
    if opposite_signs(side_a, side_b) && opposite_signs(side_c, side_d) {
        return true;
    }
    (side_a == 0.0 && in_box(c, d, a))
        || (side_b == 0.0 && in_box(c, d, b))
        || (side_c == 0.0 && in_box(a, b, c))
        || (side_d == 0.0 && in_box(a, b, d))
}

/// `point` lies inside `triangle` or on one of its sides. A triangle with
/// no area holds no point: its sides, as segments, are the caller's to
/// test.
pub fn in_triangle(point: Vec2, triangle: [Vec2; 3]) -> bool {
    let [p, q, r] = triangle;
    if orientation(p, q, r) == 0.0 {
        return false;
    }
    let sides = [
        orientation(p, q, point),
        orientation(q, r, point),
        orientation(r, p, point),
    ];
    sides.iter().all(|&side| side >= 0.0) || sides.iter().all(|&side| side <= 0.0)
}

/// Which side of the line from `from` to `to` `point` lies on: the sign of
/// the cross product, zero on the line.
fn orientation(from: Vec2, to: Vec2, point: Vec2) -> f32 {
    (to - from).cross(point - from)
}

/// One of `p` and `q` is positive and the other negative.
fn opposite_signs(p: f32, q: f32) -> bool {
    (p > 0.0 && q < 0.0) || (p < 0.0 && q > 0.0)
}

/// `point` lies in the bounding box of `a` and `b`, edges included: on the
/// segment, for a point already known to be on its line.
fn in_box(a: Vec2, b: Vec2, point: Vec2) -> bool {
    a.x.min(b.x) <= point.x
        && point.x <= a.x.max(b.x)
        && a.y.min(b.y) <= point.y
        && point.y <= a.y.max(b.y)
}
