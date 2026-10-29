//! The player's controls, as the front end hands them to the core.

use crate::geometry::Dir;

/// The controls one [`crate::Game::tick`] sees.
///
/// `dir`, `fast` and `slow` are held controls: their state at the tick.
/// `confirm` and `pause` are presses: true for the one tick that follows
/// the key press, so a press is neither lost nor repeated whatever the
/// frame rate.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct InputState {
    /// The direction held, if any.
    pub dir: Option<Dir>,
    /// The fast-draw button is held.
    pub fast: bool,
    /// The slow-draw button is held.
    pub slow: bool,
    /// Start or confirm was pressed.
    pub confirm: bool,
    /// Pause was pressed.
    pub pause: bool,
}
