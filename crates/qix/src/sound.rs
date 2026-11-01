//! The sound effects the events of the game play.

use qix_core::{DrawKind, Event};

/// A sound effect.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sound {
    /// One step of the stix, drawn with this kind.
    Draw(DrawKind),
    /// A stix claimed cells.
    Claim,
    /// The player lost a life.
    Death,
    /// The level is complete.
    LevelComplete,
    /// The player earned a life.
    ExtraLife,
}

impl Sound {
    /// Every sound effect.
    #[cfg_attr(
        not(feature = "audio"),
        allow(dead_code, reason = "only the audio feature prepares them all")
    )]
    pub const ALL: [Self; 6] = [
        Self::Draw(DrawKind::Fast),
        Self::Draw(DrawKind::Slow),
        Self::Claim,
        Self::Death,
        Self::LevelComplete,
        Self::ExtraLife,
    ];
}

/// The sound `event` plays, if any: the end of the game plays none, the
/// death raised with it does.
pub fn sound_for(event: &Event) -> Option<Sound> {
    match *event {
        Event::Draw(kind) => Some(Sound::Draw(kind)),
        Event::Claim { .. } => Some(Sound::Claim),
        Event::Death(_) => Some(Sound::Death),
        Event::ExtraLife => Some(Sound::ExtraLife),
        Event::LevelComplete { .. } => Some(Sound::LevelComplete),
        Event::GameOver => None,
    }
}

/// The sounds the events of one tick play, each once, in the order of
/// the first event playing it. The step of the stix is a background
/// tick: any other sound of the tick plays instead of it.
pub fn sounds(events: &[Event]) -> Vec<Sound> {
    let mut sounds: Vec<Sound> = Vec::new();
    for sound in events.iter().filter_map(sound_for) {
        if !sounds.contains(&sound) {
            sounds.push(sound);
        }
    }
    if sounds.iter().any(|sound| !matches!(sound, Sound::Draw(_))) {
        sounds.retain(|sound| !matches!(sound, Sound::Draw(_)));
    }
    sounds
}
