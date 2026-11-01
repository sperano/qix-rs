use eframe::{App, Storage};
use qix_core::tuning::ATTRACT_DELAY_TICKS;
use qix_core::tuning::MARKER_STEP_TICKS;
use qix_core::{DEMO_SCRIPT, Dir, DrawKind, GameState, HighScores, InputState};

use crate::app::{QixApp, save_if_changed};
use crate::persist::{HIGH_SCORES_KEY, SETTINGS_KEY, load_high_scores, load_settings};
use crate::persist_test::{MemoryStorage, entry, stored};
use crate::settings::{SCALE_RANGE, Scale, Settings};
use crate::sound::Sound;

/// The stored value of settings that only change the starting lives.
const FIVE_LIVES: &str = "(starting_lives:5)";

/// A storage holding `value` under [`SETTINGS_KEY`].
fn stored_settings(value: &str) -> MemoryStorage {
    let mut storage = MemoryStorage::default();
    storage.set_string(SETTINGS_KEY, value.to_owned());
    storage
}

/// Settings other than the default ones.
fn edited() -> Settings {
    Settings {
        scale: Scale::new(*SCALE_RANGE.end()),
        ..Settings::default()
    }
}

/// The stored value of a one-entry table.
const ONE_ENTRY: &str = r#"[(name:"QIX",score:500)]"#;

#[test]
fn the_app_saves_the_table_it_loaded() {
    let app_storage = stored(ONE_ENTRY);
    let mut app = QixApp::new(Some(&app_storage));
    let mut storage = MemoryStorage::default();

    app.save(&mut storage);

    assert_eq!(
        storage.get_string(HIGH_SCORES_KEY).as_deref(),
        Some(ONE_ENTRY)
    );
}

#[test]
fn an_app_without_storage_saves_an_empty_table() {
    let mut app = QixApp::new(None);
    let mut storage = stored(ONE_ENTRY);

    app.save(&mut storage);

    assert_eq!(load_high_scores(Some(&storage)), HighScores::default());
}

#[test]
fn an_unchanged_table_is_not_written() {
    let table = HighScores::from(vec![entry("QIX", 500)]);
    let mut saved = table.clone();
    let mut storage = MemoryStorage::default();

    save_if_changed(Some(&mut storage), &mut saved, &table);

    assert_eq!(storage.get_string(HIGH_SCORES_KEY), None);
    assert_eq!(storage.flushes(), 0);
    assert_eq!(saved, table);
}

#[test]
fn a_changed_table_is_written_flushed_and_recorded_as_saved() {
    let table = HighScores::from(vec![entry("QIX", 500)]);
    let mut saved = HighScores::default();
    let mut storage = MemoryStorage::default();

    save_if_changed(Some(&mut storage), &mut saved, &table);

    assert_eq!(
        storage.get_string(HIGH_SCORES_KEY).as_deref(),
        Some(ONE_ENTRY)
    );
    assert_eq!(storage.flushes(), 1);
    assert_eq!(storage.flushed_string(HIGH_SCORES_KEY), Some(ONE_ENTRY));
    assert_eq!(saved, table);
}

#[test]
fn a_changed_table_without_storage_is_recorded_as_saved() {
    let table = HighScores::from(vec![entry("QIX", 500)]);
    let mut saved = HighScores::default();

    save_if_changed(None, &mut saved, &table);

    assert_eq!(saved, table);
}

#[test]
fn the_app_saves_the_settings_it_loaded() {
    let app_storage = stored_settings(FIVE_LIVES);
    let mut app = QixApp::new(Some(&app_storage));
    let mut storage = MemoryStorage::default();

    app.save(&mut storage);

    assert_eq!(
        load_settings(Some(&storage)),
        load_settings(Some(&app_storage))
    );
    assert_eq!(load_settings(Some(&storage)).starting_lives.get(), 5);
}

#[test]
fn an_app_without_storage_saves_the_default_settings() {
    let mut app = QixApp::new(None);
    let mut storage = stored_settings(FIVE_LIVES);

    app.save(&mut storage);

    assert_eq!(load_settings(Some(&storage)), Settings::default());
}

#[test]
fn applied_settings_are_written_and_flushed() {
    let mut app = QixApp::new(None);
    let mut storage = MemoryStorage::default();

    app.apply_settings(edited(), Some(&mut storage));

    assert_eq!(storage.flushes(), 1);
    let flushed = storage.flushed_string(SETTINGS_KEY).map(str::to_owned);
    assert_eq!(flushed, storage.get_string(SETTINGS_KEY));
    assert_eq!(load_settings(Some(&storage)), edited());
}

#[test]
fn applied_settings_are_the_ones_the_app_saves() {
    let mut app = QixApp::new(None);
    app.apply_settings(edited(), None);
    let mut storage = MemoryStorage::default();

    app.save(&mut storage);

    assert_eq!(load_settings(Some(&storage)), edited());
}

#[test]
fn unchanged_settings_are_not_written() {
    let mut app = QixApp::new(None);
    let mut storage = MemoryStorage::default();

    app.apply_settings(Settings::default(), Some(&mut storage));

    assert_eq!(storage.get_string(SETTINGS_KEY), None);
    assert_eq!(storage.flushes(), 0);
}

#[test]
fn the_stored_rules_set_up_the_first_game() {
    let app = QixApp::new(Some(&stored_settings(FIVE_LIVES)));

    assert_eq!(app.session().state(), GameState::Title);
    assert_eq!(app.session().game().lives(), 5);
}

#[test]
fn applied_settings_set_up_the_next_game() {
    let mut app = QixApp::new(None);
    assert_ne!(app.session().game().lives(), 5);

    app.apply_settings(load_settings(Some(&stored_settings(FIVE_LIVES))), None);

    assert_eq!(app.session().state(), GameState::Title);
    assert_eq!(app.session().game().lives(), 5);
}

/// One tick of input with confirm pressed.
const CONFIRM: InputState = InputState {
    dir: None,
    fast: false,
    slow: false,
    confirm: true,
    pause: false,
};

/// Runs `ticks` ticks of `app` with no control held.
fn idle(app: &mut QixApp, ticks: u32) {
    for _ in 0..ticks {
        app.tick(&InputState::default());
    }
}

#[test]
fn an_idle_title_shows_the_demo_after_the_attract_delay() {
    let mut app = QixApp::new(None);
    idle(&mut app, ATTRACT_DELAY_TICKS - 1);
    assert!(app.attract().demo().is_none());
    assert!(std::ptr::eq(app.shown_game(), app.session().game()));

    idle(&mut app, 2);

    let demo = app.attract().demo().expect("the demo plays");
    assert!(std::ptr::eq(app.shown_game(), demo.game()));
    assert_eq!(app.shown_game().ticks(), 1);
    assert_eq!(app.session().state(), GameState::Title);
}

#[test]
fn confirm_during_the_demo_stops_it_and_starts_a_game() {
    let mut app = QixApp::new(None);
    idle(&mut app, ATTRACT_DELAY_TICKS);
    assert!(app.attract().demo().is_some());

    app.tick(&CONFIRM);

    assert!(app.attract().demo().is_none());
    assert_eq!(app.session().state(), GameState::Playing);
    assert!(std::ptr::eq(app.shown_game(), app.session().game()));
}

#[test]
fn no_demo_plays_once_a_game_is_on() {
    let mut app = QixApp::new(None);
    app.tick(&CONFIRM);
    app.tick(&InputState {
        pause: true,
        ..InputState::default()
    });
    assert_eq!(app.session().state(), GameState::Paused);

    idle(&mut app, 2 * ATTRACT_DELAY_TICKS);

    assert!(app.attract().demo().is_none());
}

#[test]
fn the_demo_ends_back_on_the_title() {
    let mut app = QixApp::new(None);
    let script: u32 = DEMO_SCRIPT.iter().map(|step| step.ticks).sum();
    idle(&mut app, ATTRACT_DELAY_TICKS + script - 1);
    assert!(app.attract().demo().is_some());

    idle(&mut app, 1);

    assert!(app.attract().demo().is_none());
    assert_eq!(app.session().state(), GameState::Title);
    assert_eq!(app.claim_fill().fresh(), None);
}

/// An app idled until the demo's first claim, checking that no fill
/// animation runs before it.
fn app_at_the_first_claim_of_the_demo() -> QixApp {
    let mut app = QixApp::new(None);
    idle(&mut app, ATTRACT_DELAY_TICKS);
    let claimed = |app: &QixApp| app.shown_game().board().claimed_fraction() > 0.0;
    let script: u32 = DEMO_SCRIPT.iter().map(|step| step.ticks).sum();
    for _ in 0..script {
        if claimed(&app) {
            break;
        }
        assert_eq!(app.claim_fill().fresh(), None);
        idle(&mut app, 1);
    }
    assert!(claimed(&app), "the demo claims nothing");
    app
}

#[test]
fn a_claim_of_the_demo_starts_the_fill_animation() {
    let app = app_at_the_first_claim_of_the_demo();

    let (runs, progress) = app.claim_fill().fresh().expect("the claim animates");
    assert!(!runs.is_empty());
    assert_eq!(progress, 0.0);
}

#[test]
fn stopping_the_demo_while_its_claim_animates_stops_the_animation() {
    let mut app = app_at_the_first_claim_of_the_demo();
    assert!(app.claim_fill().fresh().is_some());

    app.tick(&CONFIRM);

    assert!(std::ptr::eq(app.shown_game(), app.session().game()));
    assert_eq!(app.claim_fill().fresh(), None);
}

/// Holding up with the fast-draw button: from its start, the marker
/// draws a stix up into the board.
const DRAW_UP: InputState = InputState {
    dir: Some(Dir::Up),
    fast: true,
    slow: false,
    confirm: false,
    pause: false,
};

/// An app with a game started, the marker one step of the stix up.
fn app_one_step_up() -> QixApp {
    let mut app = QixApp::new(None);
    app.tick(&CONFIRM);
    for _ in 0..MARKER_STEP_TICKS {
        app.tick(&DRAW_UP);
    }
    app
}

#[test]
fn a_step_of_the_stix_queues_its_sound() {
    let app = app_one_step_up();

    assert_eq!(app.sounds(), [Sound::Draw(DrawKind::Fast)]);
}

#[test]
fn playing_the_sounds_empties_the_queue() {
    let mut app = app_one_step_up();

    app.play_sounds();

    assert_eq!(app.sounds(), []);
}

#[test]
fn a_paused_game_queues_no_sound() {
    let mut app = app_one_step_up();
    app.play_sounds();

    app.tick(&InputState {
        pause: true,
        ..DRAW_UP
    });
    idle(&mut app, MARKER_STEP_TICKS);

    assert_eq!(app.session().state(), GameState::Paused);
    assert_eq!(app.sounds(), []);
}

#[test]
fn the_demo_queues_no_sound() {
    let app = app_at_the_first_claim_of_the_demo();

    assert_eq!(app.sounds(), []);
}
