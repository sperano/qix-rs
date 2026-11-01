//! The fill animation of a claim: the cells a claim takes flash, then
//! fade into the colour of their fill.
//!
//! The core only keeps the board as it is now, so the animation finds
//! the cells a claim took by comparing the board with the one it saw
//! last.

use qix_core::{Board, Cell};

use crate::render::board::{CellRun, claimed_runs};

/// Ticks a claim's fill animation lasts: under half a second.
pub const CLAIM_FILL_TICKS: u32 = 24;

/// The cells the last claim took, while their animation runs.
#[derive(Clone, Debug)]
pub struct ClaimFill {
    /// The board as the last tick saw it.
    seen: Board,
    /// The cells the last claim took, as runs along their rows.
    fresh: Vec<CellRun>,
    /// Ticks of the animation still to run, zero when none runs.
    ticks_left: u32,
}

impl ClaimFill {
    /// No animation, on a board seen as `board`.
    pub fn new(board: &Board) -> Self {
        Self {
            seen: board.clone(),
            fresh: Vec::new(),
            ticks_left: 0,
        }
    }

    /// Runs one tick of the animation with `board` shown: cells claimed
    /// since the last tick start a new one, which replaces the one
    /// running. A board that lost claimed cells (a new level, a new
    /// game, another game shown) is not a claim: the animation stops.
    pub fn tick(&mut self, board: &Board) {
        self.ticks_left = self.ticks_left.saturating_sub(1);
        if *board == self.seen {
            return;
        }
        match claimed_since(&self.seen, board) {
            Some(gained) => {
                self.fresh = claimed_runs(&gained);
                self.ticks_left = CLAIM_FILL_TICKS;
            }
            None => {
                self.fresh.clear();
                self.ticks_left = 0;
            }
        }
        self.seen.clone_from(board);
    }

    /// The cells of the claim being animated, and how far the animation
    /// is, from 0 on the tick of the claim towards 1; `None` when no
    /// animation runs.
    pub fn fresh(&self) -> Option<(&[CellRun], f32)> {
        (self.ticks_left > 0).then(|| {
            let done = CLAIM_FILL_TICKS - self.ticks_left;
            (self.fresh.as_slice(), done as f32 / CLAIM_FILL_TICKS as f32)
        })
    }
}

/// A board holding only the cells claimed on `now` that were free on
/// `before`, or `None` when the two differ in size or `now` has lost a
/// claimed cell.
fn claimed_since(before: &Board, now: &Board) -> Option<Board> {
    let (width, height) = (now.width(), now.height());
    if (before.width(), before.height()) != (width, height) {
        return None;
    }
    let mut gained = Board::new(width, height);
    for y in 0..height {
        for x in 0..width {
            let (was, is) = (cell(before, x, y), cell(now, x, y));
            match (was, is) {
                (Cell::Free, Cell::Claimed(_)) => gained.set_cell(x, y, is),
                (Cell::Claimed(_), Cell::Free) => return None,
                _ => {}
            }
        }
    }
    Some(gained)
}

/// The cell `(x, y)` of `board`.
fn cell(board: &Board, x: u16, y: u16) -> Cell {
    board.cell(x.into(), y.into())
}
