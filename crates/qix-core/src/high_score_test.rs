use crate::tuning::HIGH_SCORE_COUNT;
use crate::{HighScore, HighScores, Initials};

/// Points between two entries of [`table_of`].
const SCORE_STEP: u32 = 1000;

/// The score of the `n`th entry of [`table_of`], best first.
fn full_score(n: usize) -> u32 {
    u32::try_from(HIGH_SCORE_COUNT - n).unwrap() * SCORE_STEP
}

fn entry(name: &str, score: u32) -> HighScore {
    HighScore {
        name: Initials::try_from(name.to_owned()).unwrap(),
        score,
    }
}

/// A table of the first `count` entries of [`full_table`].
fn table_of(count: usize) -> HighScores {
    let mut table = HighScores::default();
    for n in 0..count {
        table.insert(entry("AAA", full_score(n)));
    }
    assert_eq!(table.entries().len(), count);
    table
}

/// A table of [`HIGH_SCORE_COUNT`] entries, [`SCORE_STEP`] apart, the
/// last scoring [`SCORE_STEP`].
fn full_table() -> HighScores {
    table_of(HIGH_SCORE_COUNT)
}

fn names(table: &HighScores) -> Vec<String> {
    table.entries().iter().map(|e| e.name.to_string()).collect()
}

fn scores(table: &HighScores) -> Vec<u32> {
    table.entries().iter().map(|entry| entry.score).collect()
}

#[test]
fn an_empty_table_has_a_best_of_zero() {
    let table = HighScores::default();

    assert!(table.entries().is_empty());
    assert_eq!(table.best(), 0);
}

#[test]
fn entries_go_in_best_first() {
    let mut table = HighScores::default();

    assert_eq!(table.insert(entry("BOB", 500)), Some(0));
    assert_eq!(table.insert(entry("AMY", 900)), Some(0));
    assert_eq!(table.insert(entry("CAT", 700)), Some(1));
    assert_eq!(table.insert(entry("DAN", 100)), Some(3));

    assert_eq!(names(&table), ["AMY", "CAT", "BOB", "DAN"]);
    assert_eq!(scores(&table), [900, 700, 500, 100]);
    assert_eq!(table.best(), 900);
}

#[test]
fn an_equal_score_ranks_below_the_earlier_one() {
    let mut table = HighScores::default();
    table.insert(entry("OLD", 500));

    assert_eq!(table.insert(entry("NEW", 500)), Some(1));
    assert_eq!(table.entries()[0].name.to_string(), "OLD");
}

#[test]
fn a_score_of_zero_never_qualifies() {
    let mut table = HighScores::default();

    assert!(!table.qualifies(0));
    assert_eq!(table.insert(entry("AAA", 0)), None);
    assert!(table.entries().is_empty());
}

#[test]
fn any_score_qualifies_while_the_table_is_not_full() {
    let table = table_of(HIGH_SCORE_COUNT - 1);

    assert_eq!(table.rank(1), Some(HIGH_SCORE_COUNT - 1));
}

#[test]
fn a_full_table_keeps_only_the_best() {
    let mut table = full_table();
    let last = full_score(HIGH_SCORE_COUNT - 1);

    assert_eq!(
        table.insert(entry("NEW", last + 1)),
        Some(HIGH_SCORE_COUNT - 1)
    );

    assert_eq!(table.entries().len(), HIGH_SCORE_COUNT);
    assert_eq!(
        table.entries()[HIGH_SCORE_COUNT - 1],
        entry("NEW", last + 1)
    );
    assert!(!scores(&table).contains(&last));
}

#[test]
fn a_full_table_turns_away_a_score_no_better_than_its_last() {
    let mut table = full_table();
    let before = table.clone();
    let last = full_score(HIGH_SCORE_COUNT - 1);

    assert!(!table.qualifies(last));
    assert_eq!(table.insert(entry("NEW", last)), None);
    assert_eq!(table.insert(entry("NEW", last - 1)), None);
    assert_eq!(table, before);
}

#[test]
fn a_new_best_pushes_every_entry_down() {
    let mut table = full_table();
    let best = full_score(0) + 1;

    assert_eq!(table.insert(entry("TOP", best)), Some(0));

    let expected: Vec<_> = std::iter::once(best)
        .chain((0..HIGH_SCORE_COUNT - 1).map(full_score))
        .collect();
    assert_eq!(scores(&table), expected);
}

#[test]
fn a_table_from_a_list_is_sorted_best_first_keeping_ties_in_order() {
    let table = HighScores::from(vec![
        entry("ONE", 300),
        entry("TWO", 300),
        entry("BIG", 900),
    ]);

    assert_eq!(names(&table), ["BIG", "ONE", "TWO"]);
}

#[test]
fn a_table_from_a_long_list_keeps_only_the_best() {
    let mut list = vec![entry("LOW", 1)];
    list.extend(
        (0..HIGH_SCORE_COUNT)
            .rev()
            .map(|n| entry("AAA", full_score(n))),
    );

    let table = HighScores::from(list);

    assert_eq!(table, full_table());
}

#[test]
fn a_table_from_a_list_drops_scores_of_zero() {
    let table = HighScores::from(vec![entry("ZER", 0), entry("ONE", 1)]);

    assert_eq!(names(&table), ["ONE"]);
}

#[test]
fn a_table_turns_back_into_its_list() {
    let table = HighScores::from(vec![entry("BIG", 900), entry("SML", 100)]);

    assert_eq!(Vec::from(table), [entry("BIG", 900), entry("SML", 100)]);
}
