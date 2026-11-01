use qix_core::tuning::{HIGH_SCORE_COUNT, POINTS_PER_CELL};
use qix_core::{Dir, GameState, HighScore, HighScores, Initials, InitialsEntry, InputState};

use crate::numbers::thousands;
use crate::render::high_score::{high_score_entry, initials_row, table_rows};
use crate::render::screen::{Screen, screen};
use crate::render::screen_test::{
    CLAIMED_CELLS, LEVEL_H, MAX_GAME_TICKS, claimed_game, completed_level, level_game,
};

const CONFIRM: InputState = InputState {
    dir: None,
    fast: false,
    slow: false,
    confirm: true,
    pause: false,
};

fn entry(name: &str, score: u32) -> HighScore {
    HighScore {
        name: Initials::try_from(name.to_owned()).unwrap(),
        score,
    }
}

/// Ticks `entry` with `dir` held for one tick, then released.
fn tap(entry: &mut InitialsEntry, dir: Dir) {
    entry.tick(&InputState {
        dir: Some(dir),
        ..InputState::default()
    });
    entry.tick(&InputState::default());
}

#[test]
fn an_empty_table_has_no_rows() {
    assert!(table_rows(&HighScores::default()).is_empty());
}

#[test]
fn rows_align_their_ranks_and_scores() {
    let mut list = vec![entry("QIX", 1_234_567), entry("ABC", 50)];
    list.extend((1..HIGH_SCORE_COUNT).map(|n| entry("ZZZ", u32::try_from(n).unwrap())));
    let table = HighScores::from(list);

    let rows = table_rows(&table);

    assert_eq!(rows.len(), HIGH_SCORE_COUNT);
    assert_eq!(rows[0], " 1. QIX  1,234,567");
    assert_eq!(rows[1], " 2. ABC         50");
    assert_eq!(rows[HIGH_SCORE_COUNT - 1], "10. ZZZ          2");
}

#[test]
fn a_short_table_pads_nothing_it_does_not_need() {
    let table = HighScores::from(vec![entry("ONE", 900), entry("TWO", 10)]);

    assert_eq!(table_rows(&table), ["1. ONE  900", "2. TWO   10"]);
}

#[test]
fn the_initials_row_brackets_the_cursor() {
    let mut entry = InitialsEntry::new();
    assert_eq!(initials_row(&entry), "[A] A  A ");

    tap(&mut entry, Dir::Up);
    tap(&mut entry, Dir::Right);
    tap(&mut entry, Dir::Down);
    assert_eq!(initials_row(&entry), " B [Z] A ");
}

#[test]
fn the_entry_screen_shows_the_initials_the_rank_and_the_controls() {
    let game = claimed_game();
    let points = CLAIMED_CELLS * POINTS_PER_CELL;
    let table = HighScores::from(vec![entry("TOP", points + 1)]);

    assert_eq!(
        high_score_entry(&game, &table, &InitialsEntry::new()),
        Screen {
            heading: "NEW HIGH SCORE".to_owned(),
            table: vec!["[A] A  A ".to_owned()],
            lines: vec![
                format!("Score {points}, rank 2"),
                String::new(),
                "Up/Down: change the letter".to_owned(),
                "Left/Right: move".to_owned(),
                "Enter: next letter, then done".to_owned(),
            ],
        }
    );
}

#[test]
fn a_session_entering_initials_shows_the_entry_screen() {
    let mut session = completed_level(level_game(), LEVEL_H);
    session.tick(&CONFIRM);
    for _ in 0..MAX_GAME_TICKS {
        if session.state() == GameState::GameOver {
            break;
        }
        session.tick(&InputState::default());
    }
    session.tick(&CONFIRM);
    let GameState::HighScoreEntry(entry) = session.state() else {
        panic!("not entering initials: {:?}", session.state());
    };

    let expected = high_score_entry(session.game(), session.high_scores(), &entry);
    assert_eq!(screen(&session), Some(expected));
    assert_eq!(
        screen(&session).unwrap().lines[0],
        format!(
            "Score {}, rank 1",
            thousands(session.game().score().points())
        )
    );
}
