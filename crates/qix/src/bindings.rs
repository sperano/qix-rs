//! The keys of the six game controls, which the settings rebind.

use eframe::egui::Key;
use qix_core::Dir;
use serde::{Deserialize, Serialize};

use crate::input::RESERVED_KEYS;

/// A game control the player binds a key to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Control {
    Up,
    Down,
    Left,
    Right,
    Fast,
    Slow,
}

impl Control {
    /// Every control, in the order the settings list them; the order of
    /// the [`KeyBindings`] too.
    pub const ALL: [Self; CONTROL_COUNT] = [
        Self::Up,
        Self::Down,
        Self::Left,
        Self::Right,
        Self::Fast,
        Self::Slow,
    ];

    /// The movement controls and their directions, in priority order:
    /// when several are held, the first wins.
    pub const DIRS: [(Self, Dir); 4] = [
        (Self::Up, Dir::Up),
        (Self::Down, Dir::Down),
        (Self::Left, Dir::Left),
        (Self::Right, Dir::Right),
    ];

    /// The control's name on screen.
    pub fn name(self) -> &'static str {
        match self {
            Self::Up => "Up",
            Self::Down => "Down",
            Self::Left => "Left",
            Self::Right => "Right",
            Self::Fast => "Fast draw",
            Self::Slow => "Slow draw",
        }
    }

    /// The control that moves in `dir`.
    pub fn of_dir(dir: Dir) -> Self {
        match dir {
            Dir::Up => Self::Up,
            Dir::Down => Self::Down,
            Dir::Left => Self::Left,
            Dir::Right => Self::Right,
        }
    }

    /// The index of the control in [`Control::ALL`].
    fn index(self) -> usize {
        self as usize
    }
}

/// The number of [`Control`]s.
const CONTROL_COUNT: usize = 6;

/// The arcade-like default keys, in the order of [`Control::ALL`].
const DEFAULT_KEYS: [Key; CONTROL_COUNT] = [
    Key::ArrowUp,
    Key::ArrowDown,
    Key::ArrowLeft,
    Key::ArrowRight,
    Key::Z,
    Key::X,
];

/// A key for each [`Control`]: no two the same, none of
/// [`RESERVED_KEYS`]. Stored as one named key per control; stored keys
/// that break that rule load as the default bindings.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "StoredKeys", into = "StoredKeys")]
pub struct KeyBindings([Key; CONTROL_COUNT]);

impl Default for KeyBindings {
    fn default() -> Self {
        Self(DEFAULT_KEYS)
    }
}

impl KeyBindings {
    /// The key bound to `control`.
    pub fn key(&self, control: Control) -> Key {
        self.0[control.index()]
    }

    /// Binds `key` to `control`; the control `key` was bound to, if any,
    /// takes `control`'s old key, so no two controls share one. Returns
    /// false, changing nothing, for one of [`RESERVED_KEYS`].
    pub fn bind(&mut self, control: Control, key: Key) -> bool {
        if RESERVED_KEYS.contains(&key) {
            return false;
        }
        let old = self.key(control);
        if let Some(other) = self.0.iter_mut().find(|bound| **bound == key) {
            *other = old;
        }
        self.0[control.index()] = key;
        true
    }

    /// `keys` in the order of [`Control::ALL`], if they make valid
    /// bindings: no two the same, none reserved.
    fn checked(keys: [Key; CONTROL_COUNT]) -> Option<Self> {
        let unique = keys
            .iter()
            .enumerate()
            .all(|(i, key)| !keys[..i].contains(key));
        let free = keys.iter().all(|key| !RESERVED_KEYS.contains(key));
        (unique && free).then_some(Self(keys))
    }
}

/// [`KeyBindings`] as the storage keeps them, one named key per control.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
struct StoredKeys {
    up: Key,
    down: Key,
    left: Key,
    right: Key,
    fast: Key,
    slow: Key,
}

impl From<StoredKeys> for KeyBindings {
    /// The stored keys, or the default bindings if two are the same or
    /// one is reserved.
    fn from(keys: StoredKeys) -> Self {
        let StoredKeys {
            up,
            down,
            left,
            right,
            fast,
            slow,
        } = keys;
        Self::checked([up, down, left, right, fast, slow]).unwrap_or_default()
    }
}

impl From<KeyBindings> for StoredKeys {
    fn from(bindings: KeyBindings) -> Self {
        let [up, down, left, right, fast, slow] = bindings.0;
        Self {
            up,
            down,
            left,
            right,
            fast,
            slow,
        }
    }
}
