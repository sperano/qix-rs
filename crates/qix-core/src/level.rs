//! Levels: how hard each one is, and completing one at the claim
//! threshold with the bonus above it.

use std::ops::RangeInclusive;

use crate::tuning::{
    BONUS_PER_PERCENT, CLAIM_THRESHOLD_DEFAULT, CLAIM_THRESHOLD_RANGE, DIFFICULTY_RAMP_LEVELS,
    FIRST_LEVEL, LEVEL_SPARX, LEVELS_PER_EXTRA_SPARX, QIX_SPEED, QIX_SPEED_PER_LEVEL,
    SPARX_METER_TICKS, SPARX_METER_TICKS_PER_LEVEL, TWO_QIX_FROM_LEVEL,
};

/// Percent points in a whole board.
const PERCENT: f64 = 100.0;

/// Percent points added before rounding down the share above the
/// threshold: absorbs the `f32` rounding of a fraction lying on a whole
/// percent, such as 76 % stored a hair below `0.76`, yet stays well
/// under the share of one cell of a full board.
const PERCENT_ROUNDING_SLACK: f64 = 1e-4;

/// Qix of a level before [`TWO_QIX_FROM_LEVEL`], and from it on.
const ONE_QIX: usize = 1;
const TWO_QIX: usize = 2;

// The meter's countdowns stay positive at the end of the ramp.
const _: () = assert!(SPARX_METER_TICKS > DIFFICULTY_RAMP_LEVELS * SPARX_METER_TICKS_PER_LEVEL);

/// How hard a level is: what [`crate::Game::start_level`] sets it up
/// with.
#[derive(Clone, Debug, PartialEq)]
pub struct LevelConfig {
    /// The speeds each Qix keeps its midpoint within, in playfield units
    /// per tick.
    pub qix_speed: RangeInclusive<f32>,
    /// The Qix on the board.
    pub qix_count: usize,
    /// The Sparx on the border at the start of the level.
    pub sparx_count: usize,
    /// Ticks of each of the Sparx meter's two countdowns.
    pub meter_ticks: u32,
}

impl LevelConfig {
    /// The configuration of level `level`, counted from [`FIRST_LEVEL`]
    /// (a lower one plays as the first). Each level of the
    /// [`DIFFICULTY_RAMP_LEVELS`] after the first raises the Qix's speed
    /// by [`QIX_SPEED_PER_LEVEL`] and shortens the meter by
    /// [`SPARX_METER_TICKS_PER_LEVEL`], and every
    /// [`LEVELS_PER_EXTRA_SPARX`] of them add a Sparx; later levels are as
    /// hard as the last of the ramp. There are two Qix from
    /// [`TWO_QIX_FROM_LEVEL`] on.
    pub fn for_level(level: u32) -> Self {
        let ramp = level
            .saturating_sub(FIRST_LEVEL)
            .min(DIFFICULTY_RAMP_LEVELS);
        let speedup = ramp as f32 * QIX_SPEED_PER_LEVEL;
        let extra_sparx = (ramp / LEVELS_PER_EXTRA_SPARX) as usize;
        Self {
            qix_speed: QIX_SPEED.start() + speedup..=QIX_SPEED.end() + speedup,
            qix_count: if level >= TWO_QIX_FROM_LEVEL {
                TWO_QIX
            } else {
                ONE_QIX
            },
            sparx_count: LEVEL_SPARX + extra_sparx,
            meter_ticks: SPARX_METER_TICKS - ramp * SPARX_METER_TICKS_PER_LEVEL,
        }
    }
}

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
