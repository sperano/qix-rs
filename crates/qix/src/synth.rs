//! The waveforms of the sound effects, synthesised from tones so that
//! the game ships no audio files.

use crate::sound::Sound;
use qix_core::DrawKind;

/// Samples per second of every waveform; the audio output resamples
/// them to its own rate.
pub const SAMPLE_RATE: u32 = 44_100;

/// Peak amplitude of every tone, from 0 to 1, leaving headroom for
/// several sounds at once.
pub const AMPLITUDE: f32 = 0.25;

/// Seconds the amplitude takes to rise at the start of a tone and to
/// fall at its end, so that it starts and stops without a click.
pub const RAMP_SECS: f32 = 0.004;

/// Frequencies of the notes of the jingles, in hertz.
const C5: f32 = 523.25;
const E5: f32 = 659.25;
const G5: f32 = 783.99;
const C6: f32 = 1046.5;
const E6: f32 = 1318.5;
const G6: f32 = 1568.0;

/// The shape of one cycle of a tone.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wave {
    /// Full amplitude for the first half of the cycle, its opposite for
    /// the second.
    Square,
    /// A linear rise and fall over the cycle.
    Triangle,
}

impl Wave {
    /// The value of the wave `phase` of the way through its cycle, from
    /// 0 to 1, between -1 and 1.
    fn at(self, phase: f32) -> f32 {
        match self {
            Self::Square if phase < 0.5 => 1.0,
            Self::Square => -1.0,
            Self::Triangle => 4.0 * (phase - 0.5).abs() - 1.0,
        }
    }
}

/// A tone gliding linearly from one frequency to another.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tone {
    pub wave: Wave,
    /// Frequency at the start, in hertz.
    pub from_hz: f32,
    /// Frequency at the end, in hertz.
    pub to_hz: f32,
    /// Length in seconds.
    pub secs: f32,
}

/// A tone gliding from `from_hz` to `to_hz` over `secs` seconds.
const fn glide(wave: Wave, from_hz: f32, to_hz: f32, secs: f32) -> Tone {
    Tone {
        wave,
        from_hz,
        to_hz,
        secs,
    }
}

/// A tone holding `hz` for `secs` seconds.
const fn note(wave: Wave, hz: f32, secs: f32) -> Tone {
    glide(wave, hz, hz, secs)
}

/// A high blip for each fast step, shorter than the
/// [`MARKER_STEP_TICKS`](qix_core::tuning::MARKER_STEP_TICKS) between
/// two of them.
const DRAW_FAST: [Tone; 1] = [note(Wave::Square, 1000.0, 0.02)];

/// A lower blip for each slow step, shorter than the
/// [`SLOW_STEP_TICKS`](qix_core::tuning::SLOW_STEP_TICKS) between two
/// of them.
const DRAW_SLOW: [Tone; 1] = [note(Wave::Square, 500.0, 0.04)];

/// A rising sweep.
const CLAIM: [Tone; 1] = [glide(Wave::Triangle, 300.0, 1200.0, 0.18)];

/// A long falling sweep.
const DEATH: [Tone; 1] = [glide(Wave::Square, 900.0, 60.0, 0.8)];

/// A rising arpeggio, its last note held.
const LEVEL_COMPLETE: [Tone; 4] = [
    note(Wave::Triangle, C5, 0.12),
    note(Wave::Triangle, E5, 0.12),
    note(Wave::Triangle, G5, 0.12),
    note(Wave::Triangle, C6, 0.3),
];

/// A quick high trill.
const EXTRA_LIFE: [Tone; 4] = [
    note(Wave::Square, E6, 0.06),
    note(Wave::Square, G6, 0.06),
    note(Wave::Square, E6, 0.06),
    note(Wave::Square, G6, 0.06),
];

/// The tones `sound` plays, one after the other.
pub fn tones(sound: Sound) -> &'static [Tone] {
    match sound {
        Sound::Draw(DrawKind::Fast) => &DRAW_FAST,
        Sound::Draw(DrawKind::Slow) => &DRAW_SLOW,
        Sound::Claim => &CLAIM,
        Sound::Death => &DEATH,
        Sound::LevelComplete => &LEVEL_COMPLETE,
        Sound::ExtraLife => &EXTRA_LIFE,
    }
}

/// The mono samples of `sound` at [`SAMPLE_RATE`]: those of its
/// [`tones`] one after the other.
pub fn samples(sound: Sound) -> Vec<f32> {
    tones(sound)
        .iter()
        .copied()
        .flat_map(tone_samples)
        .collect()
}

/// The number of samples `tone` lasts at [`SAMPLE_RATE`].
fn tone_len(tone: Tone) -> usize {
    (tone.secs * SAMPLE_RATE as f32).round() as usize
}

/// The samples of `tone`, at [`AMPLITUDE`] between its ramps.
pub fn tone_samples(tone: Tone) -> Vec<f32> {
    let len = tone_len(tone);
    let mut phase = 0.0_f32;
    (0..len)
        .map(|i| {
            let progress = i as f32 / len as f32;
            let hz = tone.from_hz + (tone.to_hz - tone.from_hz) * progress;
            let value = tone.wave.at(phase) * AMPLITUDE * envelope(i, len);
            phase = (phase + hz / SAMPLE_RATE as f32).fract();
            value
        })
        .collect()
}

/// The gain of sample `i` of `len`: rising from 0 over [`RAMP_SECS`]
/// at the start, 1 in the middle, falling towards 0 over the last
/// [`RAMP_SECS`].
fn envelope(i: usize, len: usize) -> f32 {
    let ramp = RAMP_SECS * SAMPLE_RATE as f32;
    let from_start = i as f32 / ramp;
    let to_end = (len - i) as f32 / ramp;
    from_start.min(to_end).min(1.0)
}
