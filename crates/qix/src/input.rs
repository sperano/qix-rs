//! Keyboard sampling into the core's [`InputState`].

use eframe::egui::{self, Key};
use qix_core::InputState;

use crate::bindings::{Control, KeyBindings};

pub const CONFIRM_KEY: Key = Key::Enter;
pub const PAUSE_KEY: Key = Key::Escape;
pub const OVERLAY_KEY: Key = Key::F1;
/// Opens the settings from the title screen.
pub const OPEN_SETTINGS_KEY: Key = Key::F2;

/// The keys of fixed meaning, which no control may be bound to: they
/// work whatever the bindings, the settings screen included.
pub const RESERVED_KEYS: [Key; 4] = [CONFIRM_KEY, PAUSE_KEY, OVERLAY_KEY, OPEN_SETTINGS_KEY];

/// Reads one frame's controls: held keys for movement and drawing, as
/// `keys` binds them, fresh presses (not key repeats) for confirm and
/// pause.
pub fn sample(input: &egui::InputState, keys: &KeyBindings) -> InputState {
    let held = |control| input.key_down(keys.key(control));
    InputState {
        dir: Control::DIRS
            .iter()
            .find(|&&(control, _)| held(control))
            .map(|&(_, dir)| dir),
        fast: held(Control::Fast),
        slow: held(Control::Slow),
        confirm: first_press(input, CONFIRM_KEY),
        pause: first_press(input, PAUSE_KEY),
    }
}

/// Whether the settings key was pressed this frame, ignoring key
/// repeats.
pub fn settings_opened(input: &egui::InputState) -> bool {
    first_press(input, OPEN_SETTINGS_KEY)
}

/// The keys that went down this frame, in order, ignoring key repeats.
pub fn pressed_keys(input: &egui::InputState) -> Vec<Key> {
    input
        .events
        .iter()
        .filter_map(|event| match event {
            egui::Event::Key {
                key,
                pressed: true,
                repeat: false,
                ..
            } => Some(*key),
            _ => None,
        })
        .collect()
}

/// Whether the debug overlay key was pressed this frame, ignoring key
/// repeats. The overlay is the application's, not the game's, so the key
/// is not part of [`InputState`].
pub fn overlay_toggled(input: &egui::InputState) -> bool {
    first_press(input, OVERLAY_KEY)
}

/// Whether `wanted` went down this frame, ignoring key repeats.
fn first_press(input: &egui::InputState, wanted: Key) -> bool {
    pressed_keys(input).contains(&wanted)
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
