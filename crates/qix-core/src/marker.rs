//! The player's marker: where it is on the lattice and whether it is
//! drawing a stix.

use crate::board::Board;
use crate::geometry::{Dir, Point};

/// What the marker is doing.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MarkerState {
    /// On the border, moving along its edges.
    OnBorder,
    /// Off the border, drawing a stix.
    Drawing,
}

/// The player's marker.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Marker {
    pub pos: Point,
    pub state: MarkerState,
}

impl Marker {
    /// A marker on the border at the centre of `board`'s bottom edge.
    pub fn start(board: &Board) -> Self {
        Self {
            pos: Point::new(board.width() / 2, board.height()),
            state: MarkerState::OnBorder,
        }
    }

    /// Moves the marker one lattice step towards `dir` when that step
    /// follows a border edge of `board`; otherwise it stays put. Returns
    /// whether it moved.
    pub fn step_along_border(&mut self, board: &Board, dir: Dir) -> bool {
        let next = board
            .border_neighbours(self.pos)
            .find_map(|(edge_dir, next)| (edge_dir == dir).then_some(next));
        if let Some(next) = next {
            self.pos = next;
        }
        next.is_some()
    }
}
