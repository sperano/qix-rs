use eframe::egui::Key;

use crate::bindings::{Control, KeyBindings};
use crate::input::RESERVED_KEYS;

/// The keys the menus need, which no control may take.
const MENU_KEYS: [Key; 4] = [Key::Enter, Key::Escape, Key::F1, Key::F2];

/// A key no default binding uses.
const FREE_KEY: Key = Key::W;

#[test]
fn the_default_keys_are_the_arrows_z_and_x() {
    let keys = KeyBindings::default();
    let expected = [
        Key::ArrowUp,
        Key::ArrowDown,
        Key::ArrowLeft,
        Key::ArrowRight,
        Key::Z,
        Key::X,
    ];

    for (control, key) in Control::ALL.into_iter().zip(expected) {
        assert_eq!(keys.key(control), key, "{control:?}");
    }
}

#[test]
fn each_direction_has_its_control() {
    for (control, dir) in Control::DIRS {
        assert_eq!(Control::of_dir(dir), control, "{dir:?}");
    }
}

#[test]
fn binding_a_free_key_changes_only_that_control() {
    let mut keys = KeyBindings::default();

    assert!(keys.bind(Control::Fast, FREE_KEY));

    assert_eq!(keys.key(Control::Fast), FREE_KEY);
    for control in Control::ALL.into_iter().filter(|&c| c != Control::Fast) {
        assert_eq!(
            keys.key(control),
            KeyBindings::default().key(control),
            "{control:?}"
        );
    }
}

#[test]
fn binding_a_key_in_use_swaps_the_two_controls() {
    let mut keys = KeyBindings::default();
    let (up, down) = (keys.key(Control::Up), keys.key(Control::Down));

    assert!(keys.bind(Control::Up, down));

    assert_eq!(keys.key(Control::Up), down);
    assert_eq!(keys.key(Control::Down), up);
}

#[test]
fn binding_a_control_its_own_key_changes_nothing() {
    let mut keys = KeyBindings::default();

    assert!(keys.bind(Control::Slow, keys.key(Control::Slow)));

    assert_eq!(keys, KeyBindings::default());
}

#[test]
fn a_reserved_key_is_refused() {
    for key in RESERVED_KEYS {
        let mut keys = KeyBindings::default();

        assert!(!keys.bind(Control::Left, key), "{key:?}");

        assert_eq!(keys, KeyBindings::default(), "{key:?}");
    }
}

#[test]
fn the_menu_keys_are_refused() {
    for key in MENU_KEYS {
        let mut keys = KeyBindings::default();

        assert!(!keys.bind(Control::Left, key), "{key:?}");

        assert_eq!(keys, KeyBindings::default(), "{key:?}");
    }
}

#[test]
fn no_default_key_is_reserved() {
    let keys = KeyBindings::default();

    for control in Control::ALL {
        assert!(!RESERVED_KEYS.contains(&keys.key(control)), "{control:?}");
    }
}
