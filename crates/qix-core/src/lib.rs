//! Game logic of the Qix clone: board, marker, stix, claiming, enemies,
//! scoring, the game state machine and the high score table.
//!
//! The crate has no UI dependency, so every rule runs and is tested
//! headlessly; the `qix` binary renders its state with egui.

pub mod bitset;
pub mod board;
mod border;
pub mod claim;
pub mod collision;
pub mod event;
pub mod fuse;
pub mod game;
pub mod geometry;
pub mod high_score;
pub mod initials;
pub mod input;
pub mod level;
pub mod marker;
pub mod qix;
pub mod rng;
pub mod rules;
pub mod score;
mod segment;
pub mod sparx;
pub mod sparx_meter;
pub mod state;
pub mod stix;
pub mod tuning;
pub mod vec2;

#[cfg(test)]
mod bitset_test;
#[cfg(test)]
mod board_test;
#[cfg(test)]
mod border_test;
#[cfg(test)]
mod claim_test;
#[cfg(test)]
mod collision_test;
#[cfg(test)]
mod fuse_test;
#[cfg(test)]
mod game_test;
#[cfg(test)]
mod geometry_test;
#[cfg(test)]
mod high_score_test;
#[cfg(test)]
mod initials_test;
#[cfg(test)]
mod input_test;
#[cfg(test)]
mod level_test;
#[cfg(test)]
mod marker_test;
#[cfg(test)]
mod qix_test;
#[cfg(test)]
mod rng_test;
#[cfg(test)]
mod rules_test;
#[cfg(test)]
mod score_test;
#[cfg(test)]
mod segment_test;
#[cfg(test)]
mod sparx_meter_test;
#[cfg(test)]
mod sparx_test;
#[cfg(test)]
mod state_test;
#[cfg(test)]
mod stix_test;
#[cfg(test)]
mod vec2_test;

pub use bitset::BitSet;
pub use board::{Board, Cell, DrawKind};
pub use claim::{ClaimResult, CloseResult};
pub use event::{DeathCause, Event};
pub use fuse::Fuse;
pub use game::Game;
pub use geometry::{Dir, Point, in_triangle, segments_intersect};
pub use high_score::{HighScore, HighScores};
pub use initials::{Initials, InitialsEntry, InvalidInitials};
pub use input::{DirRepeat, InputState};
pub use level::{ClaimThreshold, LevelConfig};
pub use marker::{Marker, MarkerState};
pub use qix::Qix;
pub use rng::Rng;
pub use rules::{Rules, StartingLives};
pub use score::{Score, score_for_claim};
pub use sparx::{Sparx, SparxKind};
pub use sparx_meter::{MeterExpiry, SparxMeter};
pub use state::{GameState, Session};
pub use stix::Stix;
pub use vec2::Vec2;
