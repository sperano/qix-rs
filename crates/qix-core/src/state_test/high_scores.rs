//! The high score table of a [`Session`]: a game over whose score makes
//! it enters initials, which put the score in the table.

use super::{CONFIRM, DRAW_UP, H, MAX_GAME_TICKS, PAUSE, game_over, hold, level_game};
use crate::tuning::{HIGH_SCORE_COUNT, INITIALS_LEN, MARKER_STEP_TICKS};
use crate::{Dir, GameState, HighScore, HighScores, Initials, InitialsEntry, InputState, Session};

/// The score of every entry of the table a session starts with.
const OLD_SCORE: u32 = 1234;

/// Ticks the tests that expect the entry to wait run.
const IDLE_TICKS: u32 = 20;

/// Holding up, which steps the letter under the cursor.
const UP: InputState = InputState {
    dir: Some(Dir::Up),
    fast: false,
    slow: false,
    confirm: false,
    pause: false,
};

fn initials(text: &str) -> Initials {
    Initials::try_from(text.to_owned()).unwrap()
}

/// A table of [`HIGH_SCORE_COUNT`] entries all scoring `score`.
fn table_of(score: u32) -> HighScores {
    HighScores::from(vec![
        HighScore {
            name: initials("OLD"),
            score,
        };
        HIGH_SCORE_COUNT
    ])
}

/// A session over [`super::level_game`] with `table`, played until the
/// game is over: it completes the first level, then loses every life to
/// the Sparx of the next one, with points.
fn scored_game_over(table: HighScores) -> Session {
    let mut session = Session::with_high_scores(level_game(), table);
    session.tick(&CONFIRM);
    hold(&mut session, DRAW_UP, u32::from(H) * MARKER_STEP_TICKS);
    assert!(matches!(session.state(), GameState::LevelComplete { .. }));
    session.tick(&CONFIRM);
    for _ in 0..MAX_GAME_TICKS {
        if session.state() == GameState::GameOver {
            break;
        }
        session.tick(&InputState::default());
    }
    assert_eq!(session.state(), GameState::GameOver);
    assert!(
        session.game().score().points() > 0,
        "the game scored nothing"
    );
    session
}

/// Ticks `session` with confirm until the initials entry ends, and
/// checks it took one confirm per letter.
fn confirm_initials(session: &mut Session) {
    for _ in 1..INITIALS_LEN {
        session.tick(&CONFIRM);
        assert!(matches!(session.state(), GameState::HighScoreEntry(_)));
    }
    session.tick(&CONFIRM);
}

#[test]
fn a_session_starts_with_the_best_score_of_its_table() {
    let table = table_of(OLD_SCORE);
    let session = Session::with_high_scores(level_game(), table.clone());

    assert_eq!(session.state(), GameState::Title);
    assert_eq!(session.high_scores(), &table);
    assert_eq!(session.high_score(), OLD_SCORE);
}

#[test]
fn a_new_session_has_an_empty_table() {
    assert_eq!(
        Session::new(level_game()).high_scores(),
        &HighScores::default()
    );
}

#[test]
fn confirm_after_a_qualifying_game_over_enters_initials() {
    let mut session = scored_game_over(HighScores::default());
    let points = session.game().score().points();

    session.tick(&CONFIRM);

    assert_eq!(
        session.state(),
        GameState::HighScoreEntry(InitialsEntry::new())
    );
    assert_eq!(session.game().score().points(), points);
}

#[test]
fn the_entered_initials_put_the_score_in_the_table() {
    let mut session = scored_game_over(HighScores::default());
    let points = session.game().score().points();
    session.tick(&CONFIRM);

    session.tick(&UP);
    confirm_initials(&mut session);

    assert_eq!(session.state(), GameState::Title);
    let expected = HighScore {
        name: initials("BAA"),
        score: points,
    };
    assert_eq!(session.high_scores().entries(), [expected]);
    assert_eq!(session.game().score().points(), 0);
    assert_eq!(session.high_score(), points);
}

#[test]
fn the_entry_waits_for_its_initials() {
    let mut session = scored_game_over(HighScores::default());
    session.tick(&CONFIRM);
    let game = format!("{:?}", session.game());

    hold(&mut session, PAUSE, IDLE_TICKS);
    hold(&mut session, InputState::default(), IDLE_TICKS);

    assert_eq!(
        session.state(),
        GameState::HighScoreEntry(InitialsEntry::new())
    );
    assert_eq!(format!("{:?}", session.game()), game);
}

#[test]
fn a_score_off_a_full_table_goes_back_to_the_title() {
    let mut session = scored_game_over(table_of(u32::MAX));
    let before = session.high_scores().clone();

    session.tick(&CONFIRM);

    assert_eq!(session.state(), GameState::Title);
    assert_eq!(session.high_scores(), &before);
}

#[test]
fn a_game_over_without_points_skips_the_entry() {
    let (mut session, _) = game_over();
    assert_eq!(session.game().score().points(), 0);

    session.tick(&CONFIRM);

    assert_eq!(session.state(), GameState::Title);
    assert!(session.high_scores().entries().is_empty());
}
