//! Geometry of the playfield.

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
}
