use qix_core::{DeathCause, DrawKind, Event};

use crate::sound::{Sound, sound_for, sounds};

/// A claim of twelve cells drawn fast.
const CLAIM: Event = Event::Claim {
    cells: 12,
    kind: DrawKind::Fast,
};

const LEVEL_COMPLETE: Event = Event::LevelComplete {
    fraction: 0.75,
    bonus: 0,
    split: false,
};

/// One event of each kind, with the sound it plays.
const CASES: [(Event, Option<Sound>); 7] = [
    (
        Event::Draw(DrawKind::Fast),
        Some(Sound::Draw(DrawKind::Fast)),
    ),
    (
        Event::Draw(DrawKind::Slow),
        Some(Sound::Draw(DrawKind::Slow)),
    ),
    (CLAIM, Some(Sound::Claim)),
    (Event::Death(DeathCause::Sparx), Some(Sound::Death)),
    (Event::ExtraLife, Some(Sound::ExtraLife)),
    (LEVEL_COMPLETE, Some(Sound::LevelComplete)),
    (Event::GameOver, None),
];

#[test]
fn each_event_plays_its_sound() {
    for (event, sound) in CASES {
        assert_eq!(sound_for(&event), sound, "{event:?}");
    }
}

#[test]
fn a_tick_without_events_plays_nothing() {
    assert_eq!(sounds(&[]), []);
}

#[test]
fn a_step_of_the_stix_alone_plays_its_blip() {
    assert_eq!(
        sounds(&[Event::Draw(DrawKind::Slow)]),
        [Sound::Draw(DrawKind::Slow)]
    );
}

#[test]
fn any_other_sound_replaces_the_blip_of_the_step() {
    let events = [Event::Draw(DrawKind::Fast), CLAIM, LEVEL_COMPLETE];

    assert_eq!(sounds(&events), [Sound::Claim, Sound::LevelComplete]);
}

#[test]
fn a_sound_plays_once_however_many_events_raise_it() {
    let events = [CLAIM, Event::ExtraLife, Event::ExtraLife];

    assert_eq!(sounds(&events), [Sound::Claim, Sound::ExtraLife]);
}

#[test]
fn the_last_death_plays_the_death_alone() {
    let events = [Event::Death(DeathCause::Qix), Event::GameOver];

    assert_eq!(sounds(&events), [Sound::Death]);
}

#[test]
fn all_lists_every_sound_an_event_plays() {
    for sound in CASES.iter().filter_map(|(event, _)| sound_for(event)) {
        assert!(Sound::ALL.contains(&sound), "{sound:?} is not in ALL");
    }
    for sound in Sound::ALL {
        let played = CASES
            .iter()
            .any(|(event, _)| sound_for(event) == Some(sound));
        assert!(played, "no event plays {sound:?}");
    }
}

#[test]
fn all_lists_no_sound_twice() {
    for (i, sound) in Sound::ALL.iter().enumerate() {
        assert!(!Sound::ALL[..i].contains(sound), "{sound:?} twice");
    }
}
