use eframe::egui::Key;
use qix_core::{Dir, InputState};

use crate::bindings::Control;
use crate::render::settings::settings_screen;
use crate::settings::Settings;
use crate::settings_menu::{Row, SettingsMenu};

const CONFIRM: InputState = InputState {
    dir: None,
    fast: false,
    slow: false,
    confirm: true,
    pause: false,
};

/// The rows of the default settings, the first picked.
const DEFAULT_ROWS: [&str; 11] = [
    "> Claim threshold   75%",
    "  Starting lives    3",
    "  Window scale      x2.0",
    "  Up                Up",
    "  Down              Down",
    "  Left              Left",
    "  Right             Right",
    "  Fast draw         Z",
    "  Slow draw         X",
    "  Restore defaults",
    "  Back",
];

/// A menu over the default settings with the row of `control` picked.
fn menu_at(control: Control) -> SettingsMenu {
    let mut menu = SettingsMenu::new(Settings::default());
    let down = InputState {
        dir: Some(Dir::Down),
        ..InputState::default()
    };
    while menu.selected() != Row::Key(control) {
        menu.tick(&down);
        menu.tick(&InputState::default());
    }
    menu
}

#[test]
fn the_screen_lists_every_setting_with_its_value() {
    let screen = settings_screen(&SettingsMenu::new(Settings::default()));

    assert_eq!(screen.heading, "SETTINGS");
    assert_eq!(screen.table, DEFAULT_ROWS);
    assert_eq!(
        screen.lines,
        [
            "Up/Down: pick",
            "Left/Right: change",
            "Enter: rebind the key",
            "Escape: back",
        ]
    );
}

#[test]
fn the_picked_row_is_marked() {
    let screen = settings_screen(&menu_at(Control::Fast));

    assert_eq!(screen.table[0], "  Claim threshold   75%");
    assert_eq!(screen.table[7], "> Fast draw         Z");
}

#[test]
fn a_captured_key_asks_for_the_key_and_how_to_cancel() {
    let mut menu = menu_at(Control::Slow);
    menu.tick(&CONFIRM);

    let screen = settings_screen(&menu);

    assert_eq!(screen.table[8], "> Slow draw         press a key");
    assert_eq!(screen.lines, ["Escape: cancel"]);
}

#[test]
fn the_help_names_the_bound_keys() {
    let mut menu = menu_at(Control::Up);
    menu.tick(&CONFIRM);
    assert!(menu.capture(Key::W));

    let screen = settings_screen(&menu);

    assert_eq!(screen.table[3], "> Up                W");
    assert_eq!(screen.lines[0], "W/Down: pick");
}
