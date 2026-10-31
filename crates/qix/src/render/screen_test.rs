use qix_core::tuning::{MARKER_STEP_TICKS, POINTS_PER_CELL};
use qix_core::{Board, Dir, Game, InputState, Qix, Rng, Session, Vec2};

use crate::render::screen::{Screen, game_over, level_complete, paused, screen, title};

/// Seed of every game the tests play.
const SEED: u64 = 1;

/// Dimensions of the board of [`claimed_game`].
const W: u16 = 4;
const H: u16 = 2;

/// The cells the stix of [`claimed_game`] claims.
const CLAIMED_CELLS: u32 = 2;

fn press(confirm: bool, pause: bool) -> InputState {
    InputState {
        confirm,
        pause,
        ..InputState::default()
    }
}

fn new_session() -> Session {
    Session::new(Game::new(Rng::new(SEED)))
}

/// A game on a `W x H` board with a still Qix in its bottom-right cell
/// that drew a fast stix up and left from the marker's start, claiming
/// the [`CLAIMED_CELLS`] cells of the bottom row's left half.
fn claimed_game() -> Game {
    let qix = Qix::new(
        Vec2::new(3.25, 1.5),
        Vec2::new(3.75, 1.5),
        Vec2::default(),
        0.0,
    );
    let mut game = Game::with_board(Board::new(W, H), vec![qix], Vec::new(), Rng::new(SEED));
    for (dir, steps) in [(Dir::Up, 1), (Dir::Left, 2)] {
        let input = InputState {
            dir: Some(dir),
            fast: true,
            ..InputState::default()
        };
        for _ in 0..steps * MARKER_STEP_TICKS {
            game.tick(&input);
        }
    }
    assert_eq!(game.stix(), None, "the stix did not close");
    game
}

#[test]
fn a_new_session_shows_the_title_screen() {
    assert_eq!(screen(&new_session()), Some(title()));
}

#[test]
fn the_title_screen_lists_the_controls() {
    assert_eq!(
        title(),
        Screen {
            heading: "QIX".to_owned(),
            lines: vec![
                "Up/Down/Left/Right: move".to_owned(),
                "hold Z while moving: draw fast".to_owned(),
                "hold X while moving: draw slow, x2 points".to_owned(),
                "Escape: pause".to_owned(),
                "F1: debug overlay".to_owned(),
                String::new(),
                "Press Enter to start".to_owned(),
            ],
        }
    );
}

#[test]
fn playing_shows_no_screen() {
    let mut session = new_session();
    session.tick(&press(true, false));
    assert_eq!(screen(&session), None);
}

#[test]
fn a_paused_session_shows_the_pause_screen() {
    let mut session = new_session();
    session.tick(&press(true, false));
    session.tick(&press(false, true));
    assert_eq!(screen(&session), Some(paused()));
    assert_eq!(paused().lines, ["Press Escape to resume"]);
}

#[test]
fn a_completed_level_shows_the_claim_the_bonus_and_the_multiplier() {
    let mut game = Game::new(Rng::new(SEED));
    game.set_multiplier(3);
    assert_eq!(
        level_complete(&game, 0.8125, 6000, false),
        Screen {
            heading: "LEVEL 1 COMPLETE".to_owned(),
            lines: vec![
                "Claimed 81.2%".to_owned(),
                "Bonus 6000".to_owned(),
                "Multiplier x3".to_owned(),
                String::new(),
                "Press Enter for level 2".to_owned(),
            ],
        }
    );
}

#[test]
fn a_split_says_so_above_the_claim() {
    let game = Game::new(Rng::new(SEED));
    let lines = level_complete(&game, 0.5, 0, true).lines;
    assert_eq!(lines[..2], ["The Qix are split apart!", "Claimed 50.0%"]);
}

#[test]
fn game_over_shows_the_final_score() {
    let game = claimed_game();
    let points = CLAIMED_CELLS * POINTS_PER_CELL;
    assert_eq!(game.score().points(), points);
    assert_eq!(
        game_over(&game).lines,
        [
            format!("Final score {points}"),
            String::new(),
            "Press Enter to continue".to_owned(),
        ]
    );
}
