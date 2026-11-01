//! Painting the game state with egui.

pub mod board;
pub mod enemies;
pub mod high_score;
pub mod hud;
pub mod overlay;
pub mod screen;
pub mod settings;

#[cfg(test)]
mod board_test;
#[cfg(test)]
mod enemies_test;
#[cfg(test)]
mod high_score_test;
#[cfg(test)]
mod hud_test;
#[cfg(test)]
mod overlay_test;
#[cfg(test)]
mod screen_test;
#[cfg(test)]
mod settings_test;
