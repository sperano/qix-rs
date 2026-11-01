use qix_core::DrawKind;
use qix_core::tuning::{MARKER_STEP_TICKS, SLOW_STEP_TICKS, TICK_HZ};

use crate::sound::Sound;
use crate::synth::{AMPLITUDE, RAMP_SECS, SAMPLE_RATE, Tone, Wave, samples, tone_samples, tones};

/// Allowed error on the amplitude of a sample.
const EPSILON: f32 = 1e-6;

/// The most that rounding the length of a tone to whole samples adds to
/// or takes from its seconds.
const HALF_SAMPLE_SECS: f32 = 0.5 / SAMPLE_RATE as f32;

/// The number of times `samples` goes from below zero to zero or above.
fn rising_crossings(samples: &[f32]) -> usize {
    samples
        .windows(2)
        .filter(|pair| pair[0] < 0.0 && pair[1] >= 0.0)
        .count()
}

/// The seconds `sound` lasts.
fn secs(sound: Sound) -> f32 {
    samples(sound).len() as f32 / SAMPLE_RATE as f32
}

#[test]
fn every_tone_lasts_its_seconds() {
    for sound in Sound::ALL {
        for &tone in tones(sound) {
            let secs = tone_samples(tone).len() as f32 / SAMPLE_RATE as f32;
            assert!(tone.secs > 0.0, "{tone:?} is empty");
            assert!(
                (secs - tone.secs).abs() <= HALF_SAMPLE_SECS,
                "{tone:?}: {secs} s"
            );
        }
    }
}

#[test]
fn a_tone_of_twenty_milliseconds_has_882_samples() {
    let tone = Tone {
        wave: Wave::Square,
        from_hz: 1000.0,
        to_hz: 1000.0,
        secs: 0.02,
    };

    assert_eq!(tone_samples(tone).len(), 882);
}

#[test]
fn every_sound_lasts_the_seconds_of_its_tones() {
    for sound in Sound::ALL {
        let tones = tones(sound);
        let declared: f32 = tones.iter().map(|tone| tone.secs).sum();
        let rounding = tones.len() as f32 * HALF_SAMPLE_SECS;
        let secs = secs(sound);
        assert!((secs - declared).abs() <= rounding, "{sound:?}: {secs} s");
    }
}

#[test]
fn every_sample_stays_within_the_amplitude() {
    for sound in Sound::ALL {
        let peak = samples(sound)
            .iter()
            .fold(0.0_f32, |peak, s| peak.max(s.abs()));
        assert!(peak <= AMPLITUDE + EPSILON, "{sound:?} peaks at {peak}");
        assert!(peak > AMPLITUDE / 2.0, "{sound:?} is too quiet: {peak}");
    }
}

#[test]
fn every_tone_starts_silent_and_ends_near_silence() {
    let ramp = RAMP_SECS * SAMPLE_RATE as f32;
    for sound in Sound::ALL {
        for &tone in tones(sound) {
            let samples = tone_samples(tone);
            assert_eq!(samples[0], 0.0, "{tone:?}");
            let last = samples.last().copied().unwrap_or_default();
            assert!(last.abs() <= AMPLITUDE / ramp + EPSILON, "{tone:?}: {last}");
        }
    }
}

#[test]
fn a_held_note_cycles_at_its_frequency() {
    let hz = 441.0;
    let tone = Tone {
        wave: Wave::Square,
        from_hz: hz,
        to_hz: hz,
        secs: 1.0,
    };

    let crossings = rising_crossings(&tone_samples(tone));

    assert!((crossings as f32 - hz).abs() <= 1.0, "{crossings} cycles");
}

#[test]
fn a_glide_cycles_at_its_mean_frequency() {
    let tone = Tone {
        wave: Wave::Triangle,
        from_hz: 200.0,
        to_hz: 600.0,
        secs: 1.0,
    };

    let crossings = rising_crossings(&tone_samples(tone));

    assert!(
        (crossings as f32 - 400.0).abs() <= 1.0,
        "{crossings} cycles"
    );
}

#[test]
fn a_draw_blip_ends_before_the_next_step() {
    for (kind, ticks) in [
        (DrawKind::Fast, MARKER_STEP_TICKS),
        (DrawKind::Slow, SLOW_STEP_TICKS),
    ] {
        let step_secs = ticks as f32 / TICK_HZ as f32;
        let blip = secs(Sound::Draw(kind));
        assert!(
            blip < step_secs,
            "{kind:?}: {blip} s per {step_secs} s step"
        );
    }
}

#[test]
fn the_fast_blip_is_higher_than_the_slow_one() {
    let pitch = |kind| tones(Sound::Draw(kind))[0].from_hz;

    assert!(pitch(DrawKind::Fast) > pitch(DrawKind::Slow));
}
