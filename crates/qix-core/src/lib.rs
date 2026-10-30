//! Game logic of the Qix clone: board, marker, stix, claiming, enemies,
//! scoring and the game state machine.
//!
//! The crate has no UI dependency, so every rule runs and is tested
//! headlessly; the `qix` binary renders its state with egui.

pub mod game;
pub mod geometry;
pub mod input;
pub mod tuning;

#[cfg(test)]
mod game_test;

pub use game::Game;
pub use geometry::Dir;
pub use input::InputState;
