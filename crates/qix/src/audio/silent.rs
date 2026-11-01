//! The audio output of a build without the `audio` feature: the game is
//! silent, and links no audio library.

use crate::sound::Sound;

/// An audio output that plays nothing.
#[derive(Debug)]
pub struct Output;

impl Output {
    /// The silent output.
    pub fn open() -> Self {
        Self
    }

    /// Plays nothing.
    pub fn play(&mut self, _sound: Sound) {}
}
