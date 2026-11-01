//! The rules every new game is set up with, as the settings choose them:
//! the claim threshold and the lives it starts with.

use serde::{Deserialize, Serialize};

use crate::level::ClaimThreshold;
use crate::tuning::{STARTING_LIVES, STARTING_LIVES_RANGE};

/// The lives a game starts with, always in [`STARTING_LIVES_RANGE`];
/// (de)serialised as their number, clamped on the way in.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(from = "u32", into = "u32")]
pub struct StartingLives(u32);

impl StartingLives {
    /// `lives`, clamped into [`STARTING_LIVES_RANGE`].
    pub fn new(lives: u32) -> Self {
        Self(lives.clamp(*STARTING_LIVES_RANGE.start(), *STARTING_LIVES_RANGE.end()))
    }

    /// The number of lives.
    pub const fn get(self) -> u32 {
        self.0
    }
}

impl Default for StartingLives {
    /// [`STARTING_LIVES`].
    fn default() -> Self {
        Self(STARTING_LIVES)
    }
}

impl From<u32> for StartingLives {
    /// [`StartingLives::new`].
    fn from(lives: u32) -> Self {
        Self::new(lives)
    }
}

impl From<StartingLives> for u32 {
    fn from(lives: StartingLives) -> Self {
        lives.get()
    }
}

/// What [`crate::Game::restart`] sets a new game up with; the default is
/// the arcade's.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Rules {
    /// The claimed fraction of the board that completes a level.
    pub threshold: ClaimThreshold,
    /// The lives the game starts with.
    pub starting_lives: StartingLives,
}
