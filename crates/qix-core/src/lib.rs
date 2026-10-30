//! Game logic of the Qix clone: board, marker, stix, claiming, enemies,
//! scoring and the game state machine.
//!
//! The crate has no UI dependency, so every rule runs and is tested
//! headlessly; the `qix` binary renders its state with egui.

pub mod bitset;
pub mod board;
mod border;
pub mod claim;
pub mod game;
pub mod geometry;
pub mod input;
pub mod marker;
pub mod stix;
pub mod tuning;

#[cfg(test)]
mod bitset_test;
#[cfg(test)]
mod board_test;
#[cfg(test)]
mod border_test;
#[cfg(test)]
mod claim_test;
#[cfg(test)]
mod game_test;
#[cfg(test)]
mod geometry_test;
#[cfg(test)]
mod input_test;
#[cfg(test)]
mod marker_test;
#[cfg(test)]
mod stix_test;

pub use bitset::BitSet;
pub use board::{Board, Cell, DrawKind};
pub use game::Game;
pub use geometry::{Dir, Point};
pub use input::InputState;
pub use marker::{Marker, MarkerState};
pub use stix::Stix;
