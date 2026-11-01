use eframe::{App, Storage};
use qix_core::{GameState, HighScores};

use crate::app::{QixApp, save_if_changed};
use crate::persist::{HIGH_SCORES_KEY, SETTINGS_KEY, load_high_scores, load_settings};
use crate::persist_test::{MemoryStorage, entry, stored};
use crate::settings::{SCALE_RANGE, Scale, Settings};

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
