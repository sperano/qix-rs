//! The state [`Session`] moves to on the tick a claim earns an extra
//! life: [`Event::ExtraLife`] alone keeps play going, and must not hide
//! the [`Event::LevelComplete`] raised after it on the same tick.

use super::{DRAW_UP, SEED, hold, playing};
use crate::game_test::pen;
use crate::tuning::{BOARD_H, BOARD_W, CLAIM_THRESHOLD_RANGE, MARKER_STEP_TICKS};
use crate::{Board, ClaimThreshold, Dir, Event, Game, GameState, InputState, Rng, Session};

/// The column of the full board the Qix is penned in, left of every
/// stix the tests draw: each claims the board right of its stix.
const PEN_COLUMN: u16 = 1;

/// Lattice steps left of the marker's start the stix of
/// [`a_level_ending_extra_life_shows_the_completed_level`] runs up: far
/// enough that its claim reaches the lowest threshold.
const BONUS_STIX_OFFSET: u32 = 40;

/// Holding left without drawing.
const WALK_LEFT: InputState = InputState {
    dir: Some(Dir::Left),
    fast: false,
    slow: false,
    confirm: false,
    pause: false,
};

/// A full-size game with no Sparx and its Qix penned in [`PEN_COLUMN`].
fn penned_game() -> Game {
    let mut board = Board::new(BOARD_W, BOARD_H);
    let qix = pen(&mut board, PEN_COLUMN);
    Game::with_board(board, vec![qix], Vec::new(), Rng::new(SEED))
}

/// Draws a stix up from the marker with `session` until the tick that
/// raises an [`Event::ExtraLife`], and checks that one did.
fn draw_up_to_extra_life(session: &mut Session) {
    for _ in 0..u32::from(BOARD_H) * MARKER_STEP_TICKS {
        session.tick(&DRAW_UP);
        if session.game().events().contains(&Event::ExtraLife) {
            return;
        }
    }
    panic!("the stix earned no extra life");
}

#[test]
fn an_extra_life_keeps_playing() {
    let mut session = playing(penned_game());

    draw_up_to_extra_life(&mut session);

    assert!(!session.game().is_level_complete(), "the level ended");
    assert_eq!(session.state(), GameState::Playing);
}

#[test]
fn a_level_ending_extra_life_shows_the_completed_level() {
    let mut game = penned_game();
    game.set_claim_threshold(ClaimThreshold::new(*CLAIM_THRESHOLD_RANGE.start()));
    let mut session = playing(game);
    hold(
        &mut session,
        WALK_LEFT,
        BONUS_STIX_OFFSET * MARKER_STEP_TICKS,
    );

    draw_up_to_extra_life(&mut session);

    let Some(&Event::LevelComplete {
        fraction,
        bonus,
        split,
    }) = session.game().events().last()
    else {
        panic!("the level did not end: {:?}", session.game().events());
    };
    assert_eq!(
        session.state(),
        GameState::LevelComplete {
            fraction,
            bonus,
            split,
        }
    );
}
