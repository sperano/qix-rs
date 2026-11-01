use eframe::{App, Storage};
use qix_core::HighScores;

use crate::app::{QixApp, save_if_changed};
use crate::persist::{HIGH_SCORES_KEY, load_high_scores};
use crate::persist_test::{MemoryStorage, entry, stored};

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
