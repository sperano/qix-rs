//! The initials that name a high score, and their entry with the
//! direction controls after a qualifying game.

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::geometry::Dir;
use crate::input::InputState;
use crate::tuning::{INITIALS_LEN, INITIALS_REPEAT_DELAY_TICKS, INITIALS_REPEAT_TICKS};

/// The first letter initials may hold, and the one an entry starts at.
const FIRST_LETTER: u8 = b'A';

/// The last letter initials may hold.
const LAST_LETTER: u8 = b'Z';

/// Letters initials may hold, from [`FIRST_LETTER`] to [`LAST_LETTER`].
const LETTER_COUNT: u8 = LAST_LETTER - FIRST_LETTER + 1;

/// [`INITIALS_LEN`] capital letters from A to Z, stored as a string.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Initials([u8; INITIALS_LEN]);

impl Initials {
    /// Initials made of [`FIRST_LETTER`] only, where an entry starts.
    pub const FIRST: Self = Self([FIRST_LETTER; INITIALS_LEN]);

    /// The letters, first to last.
    pub fn letters(self) -> [char; INITIALS_LEN] {
        self.0.map(char::from)
    }

    /// These initials with the letter at `index` stepped `forward` (or
    /// back) by one, wrapping from Z to A and from A to Z.
    fn step_letter(mut self, index: usize, forward: bool) -> Self {
        let offset = if forward { 1 } else { LETTER_COUNT - 1 };
        let letter = &mut self.0[index];
        *letter = FIRST_LETTER + (*letter - FIRST_LETTER + offset) % LETTER_COUNT;
        self
    }
}

impl fmt::Display for Initials {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.letters()
            .iter()
            .try_for_each(|&letter| write!(f, "{letter}"))
    }
}

/// A string that is not [`INITIALS_LEN`] letters from A to Z.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InvalidInitials(pub String);

impl fmt::Display for InvalidInitials {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} is not {INITIALS_LEN} letters from A to Z", self.0)
    }
}

impl std::error::Error for InvalidInitials {}

impl TryFrom<String> for Initials {
    type Error = InvalidInitials;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        <[u8; INITIALS_LEN]>::try_from(value.as_bytes())
            .ok()
            .filter(|letters| letters.iter().all(u8::is_ascii_uppercase))
            .map(Self)
            .ok_or(InvalidInitials(value))
    }
}

impl From<Initials> for String {
    fn from(initials: Initials) -> Self {
        initials.to_string()
    }
}

/// Initials being entered, one letter at a time: up and down change the
/// letter under the cursor, left and right move the cursor, and confirm
/// moves it on, or ends the entry on the last letter.
///
/// A direction acts on the tick it is first held, then again after
/// [`INITIALS_REPEAT_DELAY_TICKS`] and every [`INITIALS_REPEAT_TICKS`]
/// after that while it stays held.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InitialsEntry {
    initials: Initials,
    cursor: usize,
    held: Option<Dir>,
    held_ticks: u32,
}

impl Default for InitialsEntry {
    fn default() -> Self {
        Self::new()
    }
}

impl InitialsEntry {
    /// An entry at [`Initials::FIRST`], the cursor on the first letter.
    pub const fn new() -> Self {
        Self {
            initials: Initials::FIRST,
            cursor: 0,
            held: None,
            held_ticks: 0,
        }
    }

    /// The initials entered so far.
    pub fn initials(&self) -> Initials {
        self.initials
    }

    /// The index of the letter the controls change.
    pub fn cursor(&self) -> usize {
        self.cursor
    }

    /// Applies one tick of `input`: the held direction, then confirm.
    /// Returns the initials once confirm is pressed on the last letter.
    pub fn tick(&mut self, input: &InputState) -> Option<Initials> {
        if let Some(dir) = self.repeat(input.dir) {
            self.apply(dir);
        }
        if !input.confirm {
            None
        } else if self.cursor + 1 == INITIALS_LEN {
            Some(self.initials)
        } else {
            self.cursor += 1;
            None
        }
    }

    /// The direction to act on this tick with `dir` held, if any.
    fn repeat(&mut self, dir: Option<Dir>) -> Option<Dir> {
        if dir != self.held {
            self.held = dir;
            self.held_ticks = 0;
            return dir;
        }
        let dir = dir?;
        self.held_ticks = self.held_ticks.saturating_add(1);
        let repeated = self.held_ticks.checked_sub(INITIALS_REPEAT_DELAY_TICKS)?;
        (repeated % INITIALS_REPEAT_TICKS == 0).then_some(dir)
    }

    /// Changes the letter or moves the cursor as `dir` asks.
    fn apply(&mut self, dir: Dir) {
        match dir {
            Dir::Up => self.initials = self.initials.step_letter(self.cursor, true),
            Dir::Down => self.initials = self.initials.step_letter(self.cursor, false),
            Dir::Left => self.cursor = self.cursor.saturating_sub(1),
            Dir::Right => self.cursor = (self.cursor + 1).min(INITIALS_LEN - 1),
        }
    }
}
