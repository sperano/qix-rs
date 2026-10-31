//! Ending a level: a claim reaching the claim threshold, or a stix
//! splitting the Qix apart, which raises the multiplier.

use super::Game;
use crate::bitset::BitSet;
use crate::claim::CloseResult;
use crate::event::Event;
use crate::tuning::{MAX_MULTIPLIER, STARTING_MULTIPLIER};

impl Game {
    /// The multiplier of the points every claim scores: from
    /// [`STARTING_MULTIPLIER`], one more for every level a stix ended by
    /// splitting the Qix apart, up to [`MAX_MULTIPLIER`].
    pub fn multiplier(&self) -> u32 {
        self.multiplier
    }

    /// Sets the multiplier of the points every claim scores, clamped to
    /// [`STARTING_MULTIPLIER`]`..=`[`MAX_MULTIPLIER`].
    pub fn set_multiplier(&mut self, multiplier: u32) {
        self.multiplier = multiplier.clamp(STARTING_MULTIPLIER, MAX_MULTIPLIER);
    }

    /// Ends the level after the stix that `close` reports, its claim
    /// already scored. A stix leaving the Qix in different areas ends it
    /// at once with no bonus, even at the threshold, and raises the
    /// multiplier by one up to [`MAX_MULTIPLIER`]; else a claim taking
    /// the board to the threshold ends it and scores the level bonus.
    /// Either raises [`Event::LevelComplete`].
    pub(super) fn end_level_after(&mut self, close: &CloseResult) {
        let fraction = self.board.claimed_fraction();
        let split = splits(&close.regions);
        let bonus = if split {
            self.multiplier = (self.multiplier + 1).min(MAX_MULTIPLIER);
            0
        } else if close.claim.is_some() && self.threshold.is_reached(fraction) {
            self.threshold.bonus(fraction)
        } else {
            return;
        };
        self.add_score(bonus);
        self.level_complete = true;
        self.events.push(Event::LevelComplete {
            fraction,
            bonus,
            split,
        });
    }
}

/// The Qix whose areas `regions` holds are not all in one: two Qix share
/// an area exactly when their sets are equal.
fn splits(regions: &[BitSet]) -> bool {
    regions.windows(2).any(|pair| pair[0] != pair[1])
}
