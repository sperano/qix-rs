//! Gameplay tuning values, kept in one place so they can be adjusted by play.

/// Core ticks per second: every [`crate::Game::tick`] advances the game by
/// `1 / TICK_HZ` seconds.
pub const TICK_HZ: u32 = 60;

/// Board width in cells; the playfield is one unit per cell.
pub const BOARD_W: u16 = 160;

/// Board height in cells; the playfield is one unit per cell.
pub const BOARD_H: u16 = 120;
