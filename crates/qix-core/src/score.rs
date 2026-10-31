//! Points: what a claim scores and the player's running total.

use crate::board::DrawKind;
use crate::claim::ClaimResult;
use crate::tuning::{EXTRA_LIFE_EVERY, MAX_LIVES, POINTS_PER_CELL, SLOW_DRAW_FACTOR};

/// The points `result` scores at `level_multiplier`: [`POINTS_PER_CELL`]
/// per claimed cell times the multiplier, times [`SLOW_DRAW_FACTOR`] for a
/// [`DrawKind::Slow`] claim. Saturates at `u32::MAX`.
pub fn score_for_claim(result: &ClaimResult, level_multiplier: u32) -> u32 {
    let factor = match result.kind {
        DrawKind::Fast => 1,
        DrawKind::Slow => SLOW_DRAW_FACTOR,
    };
    u32::try_from(result.cells)
        .unwrap_or(u32::MAX)
        .saturating_mul(POINTS_PER_CELL)
        .saturating_mul(level_multiplier)
        .saturating_mul(factor)
}

/// The extra lives a player with `lives` earns when the score goes from
/// `before` to `after`: one for every multiple of [`EXTRA_LIFE_EVERY`]
/// crossed, as many as fit below [`MAX_LIVES`].
pub fn extra_lives(before: Score, after: Score, lives: u32) -> u32 {
    let steps = (after.0 / EXTRA_LIFE_EVERY).saturating_sub(before.0 / EXTRA_LIFE_EVERY);
    steps.min(MAX_LIVES.saturating_sub(lives))
}

/// The player's points, zero at the start of a game.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Score(u32);

impl Score {
    /// The points scored so far.
    pub const fn points(self) -> u32 {
        self.0
    }

    /// Adds `points`, saturating at `u32::MAX`.
    pub fn add(&mut self, points: u32) {
        self.0 = self.0.saturating_add(points);
    }
}
