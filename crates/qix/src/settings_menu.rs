//! The settings screen's controls: a list of rows, one picked, whose
//! value the player changes, and the capture of a key to bind.
//!
//! Up and down pick a row (wrapping), left and right change a value,
//! confirm rebinds the key of a control row, or acts on the last two
//! rows; pause leaves, keeping the changes. Directions repeat as
//! [`DirRepeat`] reads them.

use eframe::egui::Key;
use qix_core::{ClaimThreshold, Dir, DirRepeat, InputState, StartingLives};

use crate::bindings::Control;
use crate::settings::{Scale, Settings};

/// How much a step changes the claim threshold, in percent points.
const THRESHOLD_STEP_PERCENT: f32 = 5.0;

/// Converts a fraction to a percentage.
const PERCENT: f32 = 100.0;

/// How much a step changes the window scale.
const SCALE_STEP: f32 = 0.5;

/// A row of the settings screen.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Row {
    Threshold,
    StartingLives,
    Scale,
    /// The key bound to a control.
    Key(Control),
    /// Puts every setting back to its default.
    Defaults,
    /// Leaves the settings screen.
    Back,
}

/// The rows of the settings screen, top to bottom.
pub const ROWS: [Row; 11] = [
    Row::Threshold,
    Row::StartingLives,
    Row::Scale,
    Row::Key(Control::ALL[0]),
    Row::Key(Control::ALL[1]),
    Row::Key(Control::ALL[2]),
    Row::Key(Control::ALL[3]),
    Row::Key(Control::ALL[4]),
    Row::Key(Control::ALL[5]),
    Row::Defaults,
    Row::Back,
];

/// The settings screen while it is open: the settings being edited.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SettingsMenu {
    settings: Settings,
    /// The index in [`ROWS`] of the row picked.
    selected: usize,
    /// Waiting for the key to bind to the picked control.
    capturing: bool,
    repeat: DirRepeat,
    /// A key was just bound: the direction held then is ignored until
    /// it is released, so binding a movement key does not also move.
    ignore_held: bool,
}

impl SettingsMenu {
    /// The screen editing `settings`, the first row picked.
    pub fn new(settings: Settings) -> Self {
        Self {
            settings,
            selected: 0,
            capturing: false,
            repeat: DirRepeat::new(),
            ignore_held: false,
        }
    }

    /// The settings as edited so far.
    pub fn settings(&self) -> &Settings {
        &self.settings
    }

    /// The row picked.
    pub fn selected(&self) -> Row {
        ROWS[self.selected]
    }

    /// Whether the screen waits for the key to bind to the picked
    /// control.
    pub fn is_capturing(&self) -> bool {
        self.capturing
    }

    /// Applies one tick of `input`; returns the edited settings when the
    /// screen closes. While a key is captured, only pause acts: it stops
    /// the capture.
    pub fn tick(&mut self, input: &InputState) -> Option<Settings> {
        if self.capturing {
            self.capturing &= !input.pause;
            return None;
        }
        if input.pause {
            return Some(self.settings);
        }
        if let Some(dir) = self.held(input.dir) {
            self.step(dir);
        }
        if input.confirm {
            return self.confirm();
        }
        None
    }

    /// Binds `key` to the control being captured and ends the capture;
    /// returns false, still capturing, for a reserved key, and when no
    /// key is being captured.
    pub fn capture(&mut self, key: Key) -> bool {
        let (true, Row::Key(control)) = (self.capturing, self.selected()) else {
            return false;
        };
        if !self.settings.keys.bind(control, key) {
            return false;
        }
        self.capturing = false;
        self.ignore_held = true;
        true
    }

    /// The direction to act on this tick with `dir` held, if any.
    fn held(&mut self, dir: Option<Dir>) -> Option<Dir> {
        if self.ignore_held && dir.is_some() {
            return None;
        }
        self.ignore_held = false;
        self.repeat.tick(dir)
    }

    /// Picks another row, or changes the value of the picked one, as
    /// `dir` asks.
    fn step(&mut self, dir: Dir) {
        match dir {
            Dir::Up => self.selected = (self.selected + ROWS.len() - 1) % ROWS.len(),
            Dir::Down => self.selected = (self.selected + 1) % ROWS.len(),
            Dir::Left => self.change(false),
            Dir::Right => self.change(true),
        }
    }

    /// Raises (or lowers) the value of the picked row by one step,
    /// within its range.
    fn change(&mut self, up: bool) {
        let sign = if up { 1.0 } else { -1.0 };
        let row = self.selected();
        let settings = &mut self.settings;
        match row {
            Row::Threshold => {
                let percent = (settings.threshold.fraction() * PERCENT).round();
                let stepped = percent + sign * THRESHOLD_STEP_PERCENT;
                settings.threshold = ClaimThreshold::new(stepped / PERCENT);
            }
            Row::StartingLives => {
                let lives = settings.starting_lives.get();
                let stepped = if up {
                    lives.saturating_add(1)
                } else {
                    lives.saturating_sub(1)
                };
                settings.starting_lives = StartingLives::new(stepped);
            }
            Row::Scale => settings.scale = Scale::new(settings.scale.get() + sign * SCALE_STEP),
            Row::Key(_) | Row::Defaults | Row::Back => {}
        }
    }

    /// Acts on confirm on the picked row; returns the settings when it
    /// leaves the screen.
    fn confirm(&mut self) -> Option<Settings> {
        match self.selected() {
            Row::Key(_) => self.capturing = true,
            Row::Defaults => self.settings = Settings::default(),
            Row::Back => return Some(self.settings),
            Row::Threshold | Row::StartingLives | Row::Scale => {}
        }
        None
    }
}
