use eframe::egui::Key;
use qix_core::tuning::{
    CLAIM_THRESHOLD_RANGE, REPEAT_DELAY_TICKS, STARTING_LIVES, STARTING_LIVES_RANGE,
};
use qix_core::{Dir, InputState};

use crate::bindings::{Control, KeyBindings};
use crate::input::CONFIRM_KEY;
use crate::settings::{SCALE_DEFAULT, SCALE_RANGE, Settings};
use crate::settings_menu::{ROWS, Row, SettingsMenu};

/// A key no default binding uses.
const FREE_KEY: Key = Key::W;

/// More steps than any value has between the ends of its range.
const MANY_STEPS: usize = 20;

const CONFIRM: InputState = InputState {
    dir: None,
    fast: false,
    slow: false,
    confirm: true,
    pause: false,
};

const PAUSE: InputState = InputState {
    dir: None,
    fast: false,
    slow: false,
    confirm: false,
    pause: true,
};

/// Holding `dir`.
fn holding(dir: Dir) -> InputState {
    InputState {
        dir: Some(dir),
        ..InputState::default()
    }
}

/// Presses `dir` once: one tick held, one released.
fn press(menu: &mut SettingsMenu, dir: Dir) {
    assert_eq!(menu.tick(&holding(dir)), None);
    assert_eq!(menu.tick(&InputState::default()), None);
}

/// A menu over the default settings with `row` picked.
fn menu_at(row: Row) -> SettingsMenu {
    let mut menu = SettingsMenu::new(Settings::default());
    while menu.selected() != row {
        press(&mut menu, Dir::Down);
    }
    menu
}

/// A menu capturing the key of `control`.
fn capturing(control: Control) -> SettingsMenu {
    let mut menu = menu_at(Row::Key(control));
    assert_eq!(menu.tick(&CONFIRM), None);
    assert!(menu.is_capturing());
    menu
}

#[test]
fn a_new_menu_picks_the_first_row_and_keeps_the_settings() {
    let menu = SettingsMenu::new(Settings::default());

    assert_eq!(menu.selected(), ROWS[0]);
    assert_eq!(menu.settings(), &Settings::default());
    assert!(!menu.is_capturing());
}

#[test]
fn every_control_has_a_row() {
    for control in Control::ALL {
        assert!(ROWS.contains(&Row::Key(control)), "{control:?}");
    }
}

#[test]
fn down_and_up_pick_the_rows_in_order_and_wrap() {
    let mut menu = SettingsMenu::new(Settings::default());

    for &row in ROWS.iter().skip(1).chain(&ROWS[..1]) {
        press(&mut menu, Dir::Down);
        assert_eq!(menu.selected(), row);
    }
    press(&mut menu, Dir::Up);
    assert_eq!(menu.selected(), ROWS[ROWS.len() - 1]);
}

#[test]
fn a_held_direction_repeats_after_the_delay() {
    let mut menu = SettingsMenu::new(Settings::default());

    for _ in 0..=REPEAT_DELAY_TICKS {
        menu.tick(&holding(Dir::Down));
    }

    assert_eq!(menu.selected(), ROWS[2]);
}

#[test]
fn right_and_left_step_the_threshold_by_five_points() {
    let mut menu = menu_at(Row::Threshold);

    press(&mut menu, Dir::Right);
    assert_eq!(menu.settings().threshold.fraction(), 0.8);
    press(&mut menu, Dir::Left);
    press(&mut menu, Dir::Left);
    assert_eq!(menu.settings().threshold.fraction(), 0.7);
}

#[test]
fn the_threshold_stays_in_its_range() {
    let mut menu = menu_at(Row::Threshold);

    (0..MANY_STEPS).for_each(|_| press(&mut menu, Dir::Right));
    assert_eq!(
        menu.settings().threshold.fraction(),
        *CLAIM_THRESHOLD_RANGE.end()
    );
    (0..MANY_STEPS).for_each(|_| press(&mut menu, Dir::Left));
    assert_eq!(
        menu.settings().threshold.fraction(),
        *CLAIM_THRESHOLD_RANGE.start()
    );
}

#[test]
fn the_starting_lives_step_by_one_within_their_range() {
    let mut menu = menu_at(Row::StartingLives);

    press(&mut menu, Dir::Right);
    assert_eq!(menu.settings().starting_lives.get(), STARTING_LIVES + 1);
    (0..MANY_STEPS).for_each(|_| press(&mut menu, Dir::Right));
    assert_eq!(
        menu.settings().starting_lives.get(),
        *STARTING_LIVES_RANGE.end()
    );
    (0..MANY_STEPS).for_each(|_| press(&mut menu, Dir::Left));
    assert_eq!(
        menu.settings().starting_lives.get(),
        *STARTING_LIVES_RANGE.start()
    );
}

#[test]
fn the_scale_steps_by_a_half_within_its_range() {
    let mut menu = menu_at(Row::Scale);

    press(&mut menu, Dir::Left);
    assert_eq!(menu.settings().scale.get(), SCALE_DEFAULT - 0.5);
    (0..MANY_STEPS).for_each(|_| press(&mut menu, Dir::Left));
    assert_eq!(menu.settings().scale.get(), *SCALE_RANGE.start());
    (0..MANY_STEPS).for_each(|_| press(&mut menu, Dir::Right));
    assert_eq!(menu.settings().scale.get(), *SCALE_RANGE.end());
}

#[test]
fn left_and_right_change_nothing_on_a_row_without_a_value() {
    for row in [Row::Key(Control::Up), Row::Defaults, Row::Back] {
        let mut menu = menu_at(row);

        press(&mut menu, Dir::Right);
        press(&mut menu, Dir::Left);

        assert_eq!(menu.settings(), &Settings::default(), "{row:?}");
    }
}

#[test]
fn confirm_on_a_control_captures_its_key() {
    let mut menu = capturing(Control::Fast);

    assert!(menu.capture(FREE_KEY));

    assert!(!menu.is_capturing());
    assert_eq!(menu.settings().keys.key(Control::Fast), FREE_KEY);
}

#[test]
fn a_reserved_key_does_not_end_the_capture() {
    let mut menu = capturing(Control::Fast);

    assert!(!menu.capture(CONFIRM_KEY));

    assert!(menu.is_capturing());
    assert_eq!(menu.settings().keys, KeyBindings::default());
}

#[test]
fn a_key_is_only_captured_after_confirm() {
    let mut menu = menu_at(Row::Key(Control::Fast));

    assert!(!menu.capture(FREE_KEY));

    assert_eq!(menu.settings().keys, KeyBindings::default());
}

#[test]
fn pause_cancels_the_capture_and_keeps_the_menu_open() {
    let mut menu = capturing(Control::Slow);

    assert_eq!(menu.tick(&PAUSE), None);

    assert!(!menu.is_capturing());
    assert_eq!(menu.settings().keys, KeyBindings::default());
}

#[test]
fn directions_and_confirm_do_nothing_while_capturing() {
    let mut menu = capturing(Control::Slow);

    menu.tick(&holding(Dir::Down));
    menu.tick(&CONFIRM);

    assert!(menu.is_capturing());
    assert_eq!(menu.selected(), Row::Key(Control::Slow));
}

#[test]
fn the_direction_held_when_a_key_is_bound_waits_for_its_release() {
    let mut menu = capturing(Control::Up);
    let down = menu.settings().keys.key(Control::Down);
    assert!(menu.capture(down));

    for _ in 0..=REPEAT_DELAY_TICKS {
        menu.tick(&holding(Dir::Up));
    }
    assert_eq!(menu.selected(), Row::Key(Control::Up));
    menu.tick(&InputState::default());

    press(&mut menu, Dir::Down);
    assert_eq!(menu.selected(), Row::Key(Control::Down));
}

#[test]
fn restore_defaults_puts_every_setting_back() {
    let mut menu = menu_at(Row::Threshold);
    press(&mut menu, Dir::Right);
    press(&mut menu, Dir::Down);
    press(&mut menu, Dir::Right);
    press(&mut menu, Dir::Down);
    press(&mut menu, Dir::Left);
    while menu.selected() != Row::Key(Control::Up) {
        press(&mut menu, Dir::Down);
    }
    assert_eq!(menu.tick(&CONFIRM), None);
    assert!(menu.capture(FREE_KEY));
    let edited = *menu.settings();
    let defaults = Settings::default();
    assert_ne!(edited.threshold, defaults.threshold);
    assert_ne!(edited.starting_lives, defaults.starting_lives);
    assert_ne!(edited.scale, defaults.scale);
    assert_ne!(edited.keys, defaults.keys);
    while menu.selected() != Row::Defaults {
        press(&mut menu, Dir::Down);
    }

    assert_eq!(menu.tick(&CONFIRM), None);

    assert_eq!(menu.settings(), &defaults);
}

#[test]
fn confirm_on_back_closes_with_the_edited_settings() {
    let mut menu = menu_at(Row::StartingLives);
    press(&mut menu, Dir::Right);
    let edited = *menu.settings();
    while menu.selected() != Row::Back {
        press(&mut menu, Dir::Down);
    }

    assert_eq!(menu.tick(&CONFIRM), Some(edited));
}

#[test]
fn pause_closes_with_the_edited_settings() {
    let mut menu = menu_at(Row::Scale);
    press(&mut menu, Dir::Right);

    assert_eq!(menu.tick(&PAUSE), Some(*menu.settings()));
    assert_ne!(*menu.settings(), Settings::default());
}

#[test]
fn confirm_on_a_value_row_does_nothing() {
    for row in [Row::Threshold, Row::StartingLives, Row::Scale] {
        let mut menu = menu_at(row);

        assert_eq!(menu.tick(&CONFIRM), None, "{row:?}");

        assert!(!menu.is_capturing(), "{row:?}");
        assert_eq!(menu.settings(), &Settings::default(), "{row:?}");
    }
}
