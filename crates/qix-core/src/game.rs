//! The game state and its fixed-step update.

use crate::board::Board;
use crate::input::InputState;
use crate::marker::{Marker, MarkerState};
use crate::tuning::{BOARD_H, BOARD_W, MARKER_STEP_TICKS};

/// The whole game, advanced one fixed step at a time.
#[derive(Debug)]
pub struct Game {
    ticks: u64,
    board: Board,
    marker: Marker,
    /// Ticks a direction has been held since the marker's last step.
    marker_held_ticks: u32,
}

impl Game {
    /// A game that has not ticked yet: a free `BOARD_W x BOARD_H` board
    /// with the marker at the centre of its bottom edge.
    pub fn new() -> Self {
        Self::with_board(Board::new(BOARD_W, BOARD_H))
    }

    /// A game on `board`, with the marker at the centre of its bottom
    /// edge.
    pub fn with_board(board: Board) -> Self {
        let marker = Marker::start(&board);
        Self {
            ticks: 0,
            board,
            marker,
            marker_held_ticks: 0,
        }
    }

    /// Advances the game by one step of `1 / TICK_HZ` seconds.
    pub fn tick(&mut self, input: &InputState) {
        self.ticks += 1;
        self.move_marker(input);
    }

    /// The number of steps run so far.
    pub fn ticks(&self) -> u64 {
        self.ticks
    }

    /// The board.
    pub fn board(&self) -> &Board {
        &self.board
    }

    /// The player's marker.
    pub fn marker(&self) -> Marker {
        self.marker
    }

    /// Steps the marker once every `MARKER_STEP_TICKS` ticks the
    /// direction is held, along border edges only.
    ///
    /// A draw button does not stop the marker from following the border;
    /// leaving it to draw a stix is not implemented yet, so a step into
    /// free space is refused with or without one.
    fn move_marker(&mut self, input: &InputState) {
        let Some(dir) = input.dir else {
            self.marker_held_ticks = 0;
            return;
        };
        self.marker_held_ticks += 1;
        if self.marker_held_ticks < MARKER_STEP_TICKS {
            return;
        }
        self.marker_held_ticks = 0;
        if self.marker.state == MarkerState::OnBorder {
            self.marker.step_along_border(&self.board, dir);
        }
    }
}

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}
