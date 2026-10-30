//! The game state and its fixed-step update.

use crate::input::InputState;

/// The whole game, advanced one fixed step at a time.
#[derive(Debug, Default)]
pub struct Game {
    ticks: u64,
}

impl Game {
    /// A game that has not ticked yet.
    pub fn new() -> Self {
        Self::default()
    }

    /// Advances the game by one step of `1 / TICK_HZ` seconds.
    pub fn tick(&mut self, _input: &InputState) {
        self.ticks += 1;
    }

    /// The number of steps run so far.
    pub fn ticks(&self) -> u64 {
        self.ticks
    }
}
