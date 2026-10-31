//! Gameplay tuning values, kept in one place so they can be adjusted by play.

use std::ops::RangeInclusive;

/// Core ticks per second: every [`crate::Game::tick`] advances the game by
/// `1 / TICK_HZ` seconds.
pub const TICK_HZ: u32 = 60;

/// Board width in cells; the playfield is one unit per cell.
pub const BOARD_W: u16 = 160;

/// Board height in cells; the playfield is one unit per cell.
pub const BOARD_H: u16 = 120;

/// Ticks between two lattice steps of the marker while a direction is
/// held.
pub const MARKER_STEP_TICKS: u32 = 2;

/// Ticks between two lattice steps of the marker drawing a slow stix: half
/// the speed of a fast one, which steps every [`MARKER_STEP_TICKS`].
pub const SLOW_STEP_TICKS: u32 = 2 * MARKER_STEP_TICKS;

/// Length of the Qix's segment, in playfield units.
pub const QIX_LENGTH: f32 = 16.0;

/// The speed of the Qix's midpoint at the first level, in playfield units
/// per tick: random jitter changes it, within these bounds.
pub const QIX_SPEED: RangeInclusive<f32> = 0.3..=0.8;

/// The most the Qix's velocity changes by in one tick, on each axis, in
/// playfield units per tick.
pub const QIX_JITTER: f32 = 0.05;

/// The fastest the Qix turns, in radians per tick either way.
pub const QIX_MAX_SPIN: f32 = 0.05;

/// The most the Qix's spin changes by in one tick, in radians per tick.
pub const QIX_SPIN_JITTER: f32 = 0.005;

/// Segments in the Qix's trail, its current one included: one per tick.
pub const QIX_TRAIL_LEN: usize = 32;

/// Half the side of the square around a drawing marker that the Qix kills
/// it in, in playfield units.
pub const MARKER_HIT_HALF_SIZE: f32 = 0.5;

/// Ticks between two lattice steps of a Sparx along the border.
pub const SPARX_STEP_TICKS: u32 = 3;

/// Ticks between two lattice steps of a Super Sparx along the stix: as
/// fast as the marker draws a fast stix, so it catches a marker that
/// stands still or draws slow.
pub const SUPER_SPARX_STEP_TICKS: u32 = MARKER_STEP_TICKS;

/// Ticks the marker may stand still on the stix it draws before the fuse
/// is lit at the stix's start: longer than a slow draw step, so drawing
/// steadily never lights it.
pub const FUSE_DELAY_TICKS: u32 = TICK_HZ / 2;

/// Ticks the lit fuse takes to burn one segment of the stix towards the
/// marker.
pub const FUSE_STEP_TICKS: u32 = 12;

/// Lives at the start of a game (the arcade's default).
pub const STARTING_LIVES: u32 = 3;

/// Ticks after a death during which the Sparx cannot kill the marker and
/// the marker cannot draw: two seconds.
pub const RESPAWN_GRACE_TICKS: u32 = 2 * TICK_HZ;

/// Points a claimed cell scores at a level multiplier of one, drawn fast.
pub const POINTS_PER_CELL: u32 = 10;

/// How many times the points of a fast claim a slow claim scores.
pub const SLOW_DRAW_FACTOR: u32 = 2;

/// The level multiplier a game starts with.
pub const STARTING_MULTIPLIER: u32 = 1;

/// The highest level multiplier: splitting the Qix no longer raises it.
pub const MAX_MULTIPLIER: u32 = 9;

/// The claimed fraction of the board that completes a level, unless the
/// settings change it.
pub const CLAIM_THRESHOLD_DEFAULT: f32 = 0.75;

/// The claim thresholds the settings may choose from; others are clamped
/// into it.
pub const CLAIM_THRESHOLD_RANGE: RangeInclusive<f32> = 0.50..=0.90;

/// Points of the level bonus for each whole percent point of the board
/// claimed above the threshold.
pub const BONUS_PER_PERCENT: u32 = 1000;

/// Ticks of each of the Sparx meter's two countdowns at the first level:
/// thirty seconds.
pub const SPARX_METER_TICKS: u32 = 30 * TICK_HZ;

/// The level a game starts at; levels count up from it.
pub const FIRST_LEVEL: u32 = 1;

/// Levels after the first over which the game gets harder: from level
/// `FIRST_LEVEL + DIFFICULTY_RAMP_LEVELS` on, every level is as hard as
/// that one.
pub const DIFFICULTY_RAMP_LEVELS: u32 = 10;

/// How much both bounds of [`QIX_SPEED`] rise with each level of the
/// ramp, in playfield units per tick: the top speed reaches one unit per
/// tick at the end of it.
pub const QIX_SPEED_PER_LEVEL: f32 = 0.02;

/// The first level with two Qix instead of one.
pub const TWO_QIX_FROM_LEVEL: u32 = 3;

/// Sparx on the border at the start of the first level.
pub const LEVEL_SPARX: usize = 2;

/// Levels of the ramp per extra Sparx at the start of a level.
pub const LEVELS_PER_EXTRA_SPARX: u32 = 2;

/// How much shorter each level of the ramp makes the Sparx meter's
/// countdowns: two seconds, down to ten at the end of it.
pub const SPARX_METER_TICKS_PER_LEVEL: u32 = 2 * TICK_HZ;

/// Sparx the Sparx meter releases at the top centre when it first runs
/// out.
pub const EXTRA_SPARX: usize = 2;
