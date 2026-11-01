//! The player's controls, as the front end hands them to the core.

use crate::board::DrawKind;
use crate::geometry::Dir;
use crate::tuning::{REPEAT_DELAY_TICKS, REPEAT_TICKS};

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

impl InputState {
    /// The kind of stix the held draw buttons draw, if any: fast when the
    /// fast button is held, whatever the slow one does.
    pub fn draw_kind(&self) -> Option<DrawKind> {
        if self.fast {
            Some(DrawKind::Fast)
        } else if self.slow {
            Some(DrawKind::Slow)
        } else {
            None
        }
    }
}

/// A held direction read as repeated presses, as the initials entry and
/// the menus read it: it acts on the tick it is first held, then again
/// after [`REPEAT_DELAY_TICKS`] and every [`REPEAT_TICKS`] after that
/// while it stays held.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DirRepeat {
    held: Option<Dir>,
    held_ticks: u32,
}

impl DirRepeat {
    /// No direction held.
    pub const fn new() -> Self {
        Self {
            held: None,
            held_ticks: 0,
        }
    }

    /// The direction to act on this tick with `dir` held, if any.
    pub fn tick(&mut self, dir: Option<Dir>) -> Option<Dir> {
        if dir != self.held {
            self.held = dir;
            self.held_ticks = 0;
            return dir;
        }
        let dir = dir?;
        self.held_ticks = self.held_ticks.saturating_add(1);
        let repeated = self.held_ticks.checked_sub(REPEAT_DELAY_TICKS)?;
        (repeated % REPEAT_TICKS == 0).then_some(dir)
    }
}
