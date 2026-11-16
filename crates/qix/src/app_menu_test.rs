use qix_core::tuning::{ATTRACT_DELAY_TICKS, CLAIM_THRESHOLD_RANGE};
use qix_core::{ClaimThreshold, Dir, GameState, InputState, StartingLives};

use crate::app::{Menu, QixApp};
use crate::app_test::{CONFIRM, idle};
use crate::playfield::Placement;
use crate::render::rules::rules_screen;
use crate::rules_book::{PAGES, RulesBook};
use crate::settings::Settings;
use crate::settings_menu::SettingsMenu;

/// One tick of input with pause pressed.
const PAUSE: InputState = InputState {
    dir: None,
    fast: false,
    slow: false,
    confirm: false,
    pause: true,
};

/// An app on the title screen with the rules open.
fn app_with_the_rules() -> QixApp {
    let mut app = QixApp::new(None);
    app.open(Menu::Rules(RulesBook::new()));
    assert_eq!(app.menu(), Some(&Menu::Rules(RulesBook::new())));
    app
}

#[test]
fn the_rules_open_over_the_title_and_stop_the_demo() {
    let mut app = QixApp::new(None);
    idle(&mut app, ATTRACT_DELAY_TICKS);
    assert!(app.attract().demo().is_some());

    app.open(Menu::Rules(RulesBook::new()));

    assert!(app.attract().demo().is_none());
    let (screen, placement) = app.screen().expect("the rules show");
    assert_eq!(
        screen,
        rules_screen(&RulesBook::new(), &Settings::default())
    );
    assert_eq!(placement, Placement::Centred);
}

#[test]
fn the_rules_do_not_open_off_the_title() {
    let mut app = QixApp::new(None);
    app.tick(&CONFIRM);

    app.open(Menu::Rules(RulesBook::new()));

    assert_eq!(app.menu(), None);
    assert_eq!(app.session().state(), GameState::Playing);
}

#[test]
fn the_rules_do_not_open_over_the_settings() {
    let mut app = QixApp::new(None);
    let settings = Menu::Settings(SettingsMenu::new(Settings::default()));
    app.open(settings);

    app.open(Menu::Rules(RulesBook::new()));

    assert_eq!(app.menu(), Some(&settings));
}

/// The starting lives of [`stricter`].
const STRICTER_LIVES: u32 = 5;

/// Settings with a threshold and starting lives other than the default
/// ones: 90% and [`STRICTER_LIVES`].
fn stricter() -> Settings {
    Settings {
        threshold: ClaimThreshold::new(*CLAIM_THRESHOLD_RANGE.end()),
        starting_lives: StartingLives::new(STRICTER_LIVES),
        ..Settings::default()
    }
}

/// Presses `dir` once in `app`: one tick held, one released.
fn press(app: &mut QixApp, dir: Dir) {
    app.tick(&InputState {
        dir: Some(dir),
        ..InputState::default()
    });
    app.tick(&InputState::default());
}

/// The text lines of the screen `app` shows.
fn shown_lines(app: &QixApp) -> Vec<String> {
    app.screen().expect("a screen shows").0.lines
}

#[test]
fn the_rules_name_the_settings_in_force() {
    let mut app = QixApp::new(None);
    app.apply_settings(stricter(), None);
    app.open(Menu::Rules(RulesBook::new()));

    assert_eq!(
        shown_lines(&app)[0],
        "Claim 90% of the field to clear a level."
    );
    for _ in 1..PAGES.len() {
        press(&mut app, Dir::Right);
    }

    let lines = shown_lines(&app);
    assert_eq!(lines[2], "Each percent over 90% adds 1,000 bonus points.");
    assert_eq!(lines[5], "You start with 5 lives and earn one more");
}

#[test]
fn confirm_in_the_rules_turns_the_page_instead_of_starting_a_game() {
    let mut app = app_with_the_rules();

    assert_eq!(app.tick(&CONFIRM), None);

    assert_eq!(app.session().state(), GameState::Title);
    let Some(Menu::Rules(book)) = app.menu() else {
        panic!("the rules close");
    };
    assert_eq!(book.page_index(), 1);
}

#[test]
fn no_demo_plays_while_the_rules_are_open() {
    let mut app = app_with_the_rules();

    idle(&mut app, 2 * ATTRACT_DELAY_TICKS);

    assert!(app.attract().demo().is_none());
    assert!(matches!(app.menu(), Some(Menu::Rules(_))));
}

#[test]
fn pause_closes_the_rules_back_to_the_title() {
    let mut app = app_with_the_rules();

    assert_eq!(app.tick(&PAUSE), None);

    assert_eq!(app.menu(), None);
    assert_eq!(app.session().state(), GameState::Title);
    assert_eq!(
        app.screen().map(|(screen, _)| screen.heading),
        Some("QIX".to_owned())
    );
}

#[test]
fn confirm_on_the_last_page_of_the_rules_closes_them() {
    let mut app = app_with_the_rules();
    for _ in 1..PAGES.len() {
        app.tick(&CONFIRM);
    }
    assert!(matches!(app.menu(), Some(Menu::Rules(book)) if book.is_last_page()));

    app.tick(&CONFIRM);

    assert_eq!(app.menu(), None);
    assert_eq!(app.session().state(), GameState::Title);
}

#[test]
fn closing_the_settings_still_returns_their_settings() {
    let mut app = QixApp::new(None);
    app.open(Menu::Settings(SettingsMenu::new(Settings::default())));

    assert_eq!(app.tick(&PAUSE), Some(Settings::default()));

    assert_eq!(app.menu(), None);
}

#[test]
fn the_settings_stay_open_on_a_tick_that_does_not_close_them() {
    let mut app = QixApp::new(None);
    app.open(Menu::Settings(SettingsMenu::new(Settings::default())));

    assert_eq!(
        app.tick(&InputState {
            dir: Some(Dir::Down),
            ..InputState::default()
        }),
        None
    );
    assert_eq!(app.tick(&InputState::default()), None);

    assert!(matches!(app.menu(), Some(Menu::Settings(_))));
}
