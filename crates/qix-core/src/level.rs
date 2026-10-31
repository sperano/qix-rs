//! Completing a level: the claim threshold and the bonus above it.

use crate::tuning::{BONUS_PER_PERCENT, CLAIM_THRESHOLD_DEFAULT, CLAIM_THRESHOLD_RANGE};

/// Percent points in a whole board.
const PERCENT: f64 = 100.0;

/// Percent points added before rounding down the share above the
/// threshold: absorbs the `f32` rounding of a fraction lying on a whole
/// percent, such as 76 % stored a hair below `0.76`, yet stays well
/// under the share of one cell of a full board.
const PERCENT_ROUNDING_SLACK: f64 = 1e-4;

/// The claimed fraction of the board that completes a level, always in
/// [`CLAIM_THRESHOLD_RANGE`].
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct ClaimThreshold(f32);

impl ClaimThreshold {
    /// The threshold `fraction`, clamped into [`CLAIM_THRESHOLD_RANGE`];
    /// a NaN gives [`CLAIM_THRESHOLD_DEFAULT`].
    pub fn new(fraction: f32) -> Self {
        if fraction.is_nan() {
            return Self::default();
        }
        Self(fraction.clamp(*CLAIM_THRESHOLD_RANGE.start(), *CLAIM_THRESHOLD_RANGE.end()))
    }

    /// The threshold, as a fraction of the board from 0 to 1.
    pub const fn fraction(self) -> f32 {
        self.0
    }

    /// Whether a board claimed to `fraction` completes the level: it is
    /// at the threshold or above.
    pub fn is_reached(self, fraction: f32) -> bool {
        fraction >= self.0
    }

    /// The bonus of a level completed with `fraction` of the board
    /// claimed: [`BONUS_PER_PERCENT`] for each whole percent point above
    /// the threshold, zero below it. Saturates at `u32::MAX`.
    pub fn bonus(self, fraction: f32) -> u32 {
        let above = (f64::from(fraction) - f64::from(self.0)) * PERCENT;
        // The cast saturates, and takes a negative share to zero.
        let percent = (above + PERCENT_ROUNDING_SLACK).floor() as u32;
        percent.saturating_mul(BONUS_PER_PERCENT)
    }
}

impl Default for ClaimThreshold {
    /// [`CLAIM_THRESHOLD_DEFAULT`].
    fn default() -> Self {
        Self(CLAIM_THRESHOLD_DEFAULT)
    }
}
