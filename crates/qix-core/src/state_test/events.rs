//! [`Session::events`]: the events of the game tick a session tick ran,
//! and none from a tick that did not tick the game.

use super::{CONFIRM, DRAW_UP, PAUSE, completed_level, level_game, playing};
use crate::tuning::MARKER_STEP_TICKS;
use crate::{DrawKind, Event, GameState, InputState, Session};

#[test]
fn a_new_session_has_no_events() {
    assert_eq!(Session::new(level_game()).events(), []);
}

#[test]
fn a_playing_tick_reports_the_events_of_the_game() {
    let mut session = playing(level_game());

    for _ in 0..MARKER_STEP_TICKS {
        session.tick(&DRAW_UP);
        assert_eq!(session.events(), session.game().events());
    }
    assert_eq!(session.events(), [Event::Draw(DrawKind::Fast)]);
}

#[test]
fn the_tick_that_pauses_reports_no_events() {
    let mut session = playing(level_game());
    for _ in 0..MARKER_STEP_TICKS {
        session.tick(&DRAW_UP);
    }
    assert!(!session.events().is_empty(), "the stix did not step");

    session.tick(&PAUSE);

    assert_eq!(session.state(), GameState::Paused);
    assert_eq!(session.events(), []);
    assert!(!session.game().events().is_empty());
}

#[test]
fn the_ticks_after_a_completed_level_report_no_events() {
    let mut session = completed_level();
    assert!(
        matches!(session.events().last(), Some(Event::LevelComplete { .. })),
        "{:?}",
        session.events()
    );

    session.tick(&InputState::default());

    assert!(matches!(session.state(), GameState::LevelComplete { .. }));
    assert_eq!(session.events(), []);
}

#[test]
fn the_confirm_that_starts_the_next_level_reports_no_events() {
    let mut session = completed_level();

    session.tick(&CONFIRM);

    assert_eq!(session.state(), GameState::Playing);
    assert_eq!(session.events(), []);
}
