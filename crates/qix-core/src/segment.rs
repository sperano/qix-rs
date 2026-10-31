//! The cells a straight segment in playfield units passes through.

use crate::vec2::Vec2;

/// The cells, as `(x, y)`, that the segment from `from` to `to` passes
/// through, in order from the cell of `from` to the cell of `to`.
///
/// Consecutive cells share a side: a segment through a lattice point
/// crosses into one of the two cells beside the diagonal step, not both.
/// The coordinates are not bounded by any board.
pub fn cells(from: Vec2, to: Vec2) -> Cells {
    Cells {
        x: Axis::new(from.x, to.x),
        y: Axis::new(from.y, to.y),
        done: false,
    }
}

/// The iterator [`cells`] returns.
#[derive(Clone, Debug)]
pub struct Cells {
    x: Axis,
    y: Axis,
    done: bool,
}

impl Iterator for Cells {
    type Item = (i32, i32);

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }
        let cell = (self.x.cell, self.y.cell);
        // An axis that is already in the end cell never steps, so rounding
        // in the crossing times cannot walk past the end.
        if self.x.left == 0 && self.y.left == 0 {
            self.done = true;
        } else if self.y.left == 0 || (self.x.left > 0 && self.x.next < self.y.next) {
            self.x.advance();
        } else {
            self.y.advance();
        }
        Some(cell)
    }
}

/// The walk along one axis: the current cell coordinate and when, as a
/// fraction of the segment, the next cell boundary is crossed.
#[derive(Clone, Debug)]
struct Axis {
    cell: i32,
    /// `1` or `-1` towards the end, `0` along a segment with no extent on
    /// this axis.
    step: i32,
    /// The fraction of the segment at which the next boundary lies.
    next: f32,
    /// The fraction of the segment between two boundaries.
    delta: f32,
    /// The boundaries left to cross.
    left: u32,
}

impl Axis {
    fn new(from: f32, to: f32) -> Self {
        let cell = from.floor();
        let extent = to - from;
        let (step, to_boundary) = if extent > 0.0 {
            (1, cell + 1.0 - from)
        } else if extent < 0.0 {
            (-1, from - cell)
        } else {
            (0, f32::INFINITY)
        };
        let delta = 1.0 / extent.abs();
        Self {
            cell: cell as i32,
            step,
            next: to_boundary * delta,
            delta,
            left: (to.floor() as i32).abs_diff(cell as i32),
        }
    }

    fn advance(&mut self) {
        self.cell += self.step;
        self.next += self.delta;
        self.left -= 1;
    }
}
