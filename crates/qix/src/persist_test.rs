use std::collections::HashMap;

use eframe::Storage;
use qix_core::tuning::HIGH_SCORE_COUNT;
use qix_core::{HighScore, HighScores, Initials};

use crate::persist::{HIGH_SCORES_KEY, load_high_scores, save_high_scores};

/// Points between two entries of [`full_table`].
const SCORE_STEP: u32 = 1500;

/// A [`Storage`] in memory.
#[derive(Default)]
pub(crate) struct MemoryStorage(HashMap<String, String>);

impl Storage for MemoryStorage {
    fn get_string(&self, key: &str) -> Option<String> {
        self.0.get(key).cloned()
    }

    fn set_string(&mut self, key: &str, value: String) {
        self.0.insert(key.to_owned(), value);
    }

    fn remove_string(&mut self, key: &str) {
        self.0.remove(key);
    }

    fn flush(&mut self) {}
}

pub(crate) fn entry(name: &str, score: u32) -> HighScore {
    HighScore {
        name: Initials::try_from(name.to_owned()).unwrap(),
        score,
    }
}

/// A table of [`HIGH_SCORE_COUNT`] entries with distinct names and
/// scores.
fn full_table() -> HighScores {
    let mut table = HighScores::default();
    for (n, letter) in ('A'..).take(HIGH_SCORE_COUNT).enumerate() {
        let score = SCORE_STEP * (u32::try_from(n).unwrap() + 1);
        table.insert(entry(&letter.to_string().repeat(3), score));
    }
    table
}

/// A storage holding `value` under [`HIGH_SCORES_KEY`].
pub(crate) fn stored(value: &str) -> MemoryStorage {
    let mut storage = MemoryStorage::default();
    storage.set_string(HIGH_SCORES_KEY, value.to_owned());
    storage
}

#[test]
fn a_saved_table_loads_back_the_same() {
    let table = full_table();
    let mut storage = MemoryStorage::default();

    save_high_scores(&mut storage, &table);

    assert_eq!(load_high_scores(Some(&storage)), table);
}

#[test]
fn an_empty_table_round_trips() {
    let mut storage = MemoryStorage::default();

    save_high_scores(&mut storage, &HighScores::default());

    assert_eq!(load_high_scores(Some(&storage)), HighScores::default());
}

#[test]
fn the_table_is_stored_as_a_readable_list() {
    let mut storage = MemoryStorage::default();
    let table = HighScores::from(vec![entry("QIX", 12_345), entry("ABC", 10)]);

    save_high_scores(&mut storage, &table);

    assert_eq!(
        storage.get_string(HIGH_SCORES_KEY).as_deref(),
        Some(r#"[(name:"QIX",score:12345),(name:"ABC",score:10)]"#)
    );
}

#[test]
fn saving_replaces_the_stored_table() {
    let mut storage = MemoryStorage::default();
    save_high_scores(&mut storage, &full_table());
    let table = HighScores::from(vec![entry("NEW", 1)]);

    save_high_scores(&mut storage, &table);

    assert_eq!(load_high_scores(Some(&storage)), table);
}

#[test]
fn without_storage_or_a_stored_table_the_table_is_empty() {
    assert_eq!(load_high_scores(None), HighScores::default());
    let storage = MemoryStorage::default();
    assert_eq!(load_high_scores(Some(&storage)), HighScores::default());
}

#[test]
fn a_table_that_does_not_decode_loads_empty() {
    for value in ["", "not ron", "(name:\"ABC\",score:1)", "[(name:\"ABC\""] {
        let storage = stored(value);
        assert_eq!(
            load_high_scores(Some(&storage)),
            HighScores::default(),
            "{value:?}"
        );
    }
}

#[test]
fn an_invalid_entry_is_dropped_and_the_others_kept() {
    for bad in [
        r#"(name:"toolong",score:1)"#,
        r#"(name:"abc",score:1)"#,
        r#"(name:"A1C",score:1)"#,
        r#"(name:"ABC",score:-1)"#,
        r#"(name:"ABC",score:4294967296)"#,
        r#"(name:"ABC")"#,
        r#""ABC""#,
        "7",
    ] {
        let storage = stored(&format!(
            r#"[(name:"TOP",score:900),{bad},(name:"LOW",score:100)]"#
        ));
        assert_eq!(
            load_high_scores(Some(&storage)),
            HighScores::from(vec![entry("TOP", 900), entry("LOW", 100)]),
            "{bad:?}"
        );
    }
}

#[test]
fn a_table_edited_out_of_order_loads_sorted_and_cut() {
    let mut list: Vec<_> = full_table().entries().iter().rev().copied().collect();
    list.push(entry("LOW", 1));
    let value = list
        .iter()
        .map(|e| format!(r#"(name:"{}",score:{})"#, e.name, e.score))
        .collect::<Vec<_>>()
        .join(",");
    let storage = stored(&format!("[{value}]"));

    assert_eq!(load_high_scores(Some(&storage)), full_table());
}
