//! Converts frame times into a whole number of fixed core ticks.

use qix_core::tuning::TICK_HZ;

/// Most ticks one frame may run. After a stall the rest of the backlog is
/// dropped rather than caught up, so slow frames cannot snowball.
pub const MAX_TICKS_PER_FRAME: u32 = 5;

/// Fraction of a tick by which a nearly complete tick still counts as
/// complete, so a frame time such as `1.0 / 60.0` that rounds below the
/// exact tick period in `f32` still yields one tick per frame.
const TICK_SNAP: f64 = 1e-3;

/// Accumulates frame time and releases it as fixed ticks.
#[derive(Debug, Default)]
pub struct FixedTimestep {
    /// Part of a tick owed but not run yet, in ticks.
    owed: f64,
}

impl FixedTimestep {
    /// Adds a frame of `dt` seconds and returns how many ticks to run now.
    pub fn advance(&mut self, dt: f32) -> u32 {
        // `max` also turns a NaN frame time into zero.
        let owed = self.owed + f64::from(dt.max(0.0)) * f64::from(TICK_HZ);
        let whole = (owed + TICK_SNAP).floor();
        self.owed = (owed - whole).max(0.0);
        // `whole` is a non-negative integer, so the cast only drops `.0`.
        whole.min(f64::from(MAX_TICKS_PER_FRAME)) as u32
    }
}
