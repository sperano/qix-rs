//! Keyboard sampling into the core's [`InputState`].

use eframe::egui::{self, Key};
use qix_core::{Dir, InputState};

/// Movement keys in priority order: when several are held, the first wins.
pub const DIR_KEYS: [(Key, Dir); 4] = [
    (Key::ArrowUp, Dir::Up),
    (Key::ArrowDown, Dir::Down),
    (Key::ArrowLeft, Dir::Left),
    (Key::ArrowRight, Dir::Right),
];
pub const FAST_KEY: Key = Key::Z;
pub const SLOW_KEY: Key = Key::X;
pub const CONFIRM_KEY: Key = Key::Enter;
pub const PAUSE_KEY: Key = Key::Escape;
pub const OVERLAY_KEY: Key = Key::F1;

/// Reads one frame's controls: held keys for movement and drawing, fresh
/// presses (not key repeats) for confirm and pause.
pub fn sample(input: &egui::InputState) -> InputState {
    InputState {
        dir: DIR_KEYS
            .iter()
            .find(|(key, _)| input.key_down(*key))
            .map(|&(_, dir)| dir),
        fast: input.key_down(FAST_KEY),
        slow: input.key_down(SLOW_KEY),
        confirm: first_press(input, CONFIRM_KEY),
        pause: first_press(input, PAUSE_KEY),
    }
}

/// Whether the debug overlay key was pressed this frame, ignoring key
/// repeats. The overlay is the application's, not the game's, so the key
/// is not part of [`InputState`].
pub fn overlay_toggled(input: &egui::InputState) -> bool {
    first_press(input, OVERLAY_KEY)
}

/// Whether `wanted` went down this frame, ignoring key repeats.
fn first_press(input: &egui::InputState, wanted: Key) -> bool {
    input.events.iter().any(|event| {
        matches!(
            event,
            egui::Event::Key { key, pressed: true, repeat: false, .. } if *key == wanted
        )
    })
}

/// Turns per-frame samples into per-tick input: a press made in a frame
/// that runs no tick waits for the next tick, and a frame that runs
/// several ticks delivers it to the first one only.
#[derive(Debug, Default)]
pub struct InputLatch {
    pending: InputState,
}

impl InputLatch {
    /// Records a frame's sample: held controls replace the previous ones,
    /// presses add up until a tick takes them.
    pub fn record(&mut self, frame: InputState) {
        self.pending = InputState {
            confirm: self.pending.confirm || frame.confirm,
            pause: self.pending.pause || frame.pause,
            ..frame
        };
    }

    /// The input for the next tick; clears the presses it delivers.
    pub fn next_tick(&mut self) -> InputState {
        let input = self.pending;
        self.pending.confirm = false;
        self.pending.pause = false;
        input
    }
}
