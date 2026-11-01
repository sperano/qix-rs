//! The audio output of the `audio` feature: kira on the default device.

use std::fmt;

use kira::sound::static_sound::{StaticSoundData, StaticSoundSettings};
use kira::{AudioManager, AudioManagerSettings, DefaultBackend, Frame};

use crate::sound::Sound;
use crate::synth::{self, SAMPLE_RATE};

/// The default audio device, with every sound effect synthesised.
pub struct Output {
    /// The device, or `None` when it could not be opened: the game then
    /// plays silently.
    manager: Option<AudioManager<DefaultBackend>>,
    /// Each sound effect and its samples.
    sounds: Vec<(Sound, StaticSoundData)>,
}

impl Output {
    /// The default audio device with every sound effect synthesised, or
    /// a silent output, after saying why on stderr, when there is none.
    pub fn open() -> Self {
        let manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())
            .inspect_err(|err| eprintln!("qix: no audio output, playing silently: {err}"))
            .ok();
        let sounds = Sound::ALL
            .iter()
            .map(|&sound| (sound, sound_data(sound)))
            .collect();
        Self { manager, sounds }
    }

    /// Starts `sound` over whatever is playing.
    pub fn play(&mut self, sound: Sound) {
        let Some(manager) = &mut self.manager else {
            return;
        };
        let Some((_, data)) = self.sounds.iter().find(|(known, _)| *known == sound) else {
            return;
        };
        // Past the device's capacity of sounds playing at once, the sound
        // is dropped: a missed blip is better than an error.
        let _ = manager.play(data.clone());
    }
}

impl fmt::Debug for Output {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Output")
            .field("open", &self.manager.is_some())
            .finish_non_exhaustive()
    }
}

/// The samples of `sound`, ready to play.
fn sound_data(sound: Sound) -> StaticSoundData {
    StaticSoundData {
        sample_rate: SAMPLE_RATE,
        frames: synth::samples(sound)
            .into_iter()
            .map(Frame::from_mono)
            .collect(),
        settings: StaticSoundSettings::default(),
        slice: None,
    }
}
