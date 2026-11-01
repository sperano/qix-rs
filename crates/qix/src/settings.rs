//! The player's settings: the rules of a new game, the key bindings and
//! the window scale, kept in eframe's storage next to the high scores.

use std::ops::RangeInclusive;

use eframe::egui::{Vec2, vec2};
use qix_core::{ClaimThreshold, Rules, StartingLives};
use serde::{Deserialize, Serialize};

use crate::bindings::KeyBindings;

/// Inner size of the window at a scale of one, in points, at the ratio
/// of the playfield and the HUD above it.
pub const BASE_WINDOW_SIZE: [f32; 2] = [480.0, 396.0];

/// The window scale of a first run.
pub const SCALE_DEFAULT: f32 = 2.0;

/// The window scales the settings may choose from; others are clamped
/// into it.
pub const SCALE_RANGE: RangeInclusive<f32> = 1.0..=3.0;

/// Everything the settings screen edits. A field missing from the
/// storage loads as its default; the rules and the scale are clamped
/// into their ranges and broken key bindings load as the default ones.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    /// The claimed fraction of the board that completes a level.
    pub threshold: ClaimThreshold,
    /// The lives a game starts with.
    pub starting_lives: StartingLives,
    /// The keys of the game controls.
    pub keys: KeyBindings,
    /// The size of the window, as a multiple of [`BASE_WINDOW_SIZE`].
    pub scale: Scale,
}

impl Settings {
    /// The rules a new game is set up with.
    pub fn rules(&self) -> Rules {
        Rules {
            threshold: self.threshold,
            starting_lives: self.starting_lives,
        }
    }
}

/// The size of the window, as a multiple of [`BASE_WINDOW_SIZE`], always
/// in [`SCALE_RANGE`]; (de)serialised as its factor, clamped on the way
/// in.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(from = "f32", into = "f32")]
pub struct Scale(f32);

impl Scale {
    /// The scale `factor`, clamped into [`SCALE_RANGE`]; a NaN gives
    /// [`SCALE_DEFAULT`].
    pub fn new(factor: f32) -> Self {
        if factor.is_nan() {
            return Self::default();
        }
        Self(factor.clamp(*SCALE_RANGE.start(), *SCALE_RANGE.end()))
    }

    /// The factor.
    pub const fn get(self) -> f32 {
        self.0
    }

    /// The inner size of the window at this scale, in points.
    pub fn window_size(self) -> Vec2 {
        vec2(BASE_WINDOW_SIZE[0], BASE_WINDOW_SIZE[1]) * self.0
    }
}

impl Default for Scale {
    /// [`SCALE_DEFAULT`].
    fn default() -> Self {
        Self(SCALE_DEFAULT)
    }
}

impl From<f32> for Scale {
    /// [`Scale::new`].
    fn from(factor: f32) -> Self {
        Self::new(factor)
    }
}

impl From<Scale> for f32 {
    fn from(scale: Scale) -> Self {
        scale.get()
    }
}
