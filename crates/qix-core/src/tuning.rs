//! Gameplay tuning values, kept in one place so they can be adjusted by play.

/// Core ticks per second: every [`crate::Game::tick`] advances the game by
/// `1 / TICK_HZ` seconds.
pub const TICK_HZ: u32 = 60;

/// Playfield width in cells.
pub const PLAYFIELD_WIDTH: u16 = 160;

/// Playfield height in cells.
pub const PLAYFIELD_HEIGHT: u16 = 120;
