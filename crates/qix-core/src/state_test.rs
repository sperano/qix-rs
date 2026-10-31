use crate::qix_test::qix_at;
use crate::tuning::{BONUS_PER_PERCENT, FIRST_LEVEL, MARKER_STEP_TICKS, STARTING_LIVES};
use crate::{
    Board, Cell, ClaimThreshold, Dir, DrawKind, Event, Game, GameState, InputState, Point, Rng,
    Session, Sparx, SparxKind,
};

/// Seed of every game the tests play.
const SEED: u64 = 1206;

/// Dimensions of the board of [`level_game`]: a stix up column `W / 2`
/// from the marker's start claims the cells right of it, leaving the Qix
/// in cell `(0, 0)` on its left.
const W: u16 = 8;
const H: u16 = 3;

/// Cells [`level_game`] claims before the stix, clear of its column and
/// of the top row: with the stix, 18 cells of 24.
const CLAIMED_BEFORE: [(u16, u16); 6] = [(0, 1), (1, 1), (2, 1), (0, 2), (1, 2), (2, 2)];

/// The board's claimed fraction once the stix of [`level_game`] closes.
const FRACTION_AFTER: f32 = 0.75;

/// The claim threshold of [`level_game`], and the whole percent points
/// [`FRACTION_AFTER`] is above it.
const LOW_THRESHOLD: f32 = 0.5;
const PERCENT_ABOVE_LOW: u32 = 25;

/// Dimensions of the board of [`sparx_game`]: its border is one loop the
/// Sparx keeps walking.
const SPARX_W: u16 = 4;
const SPARX_H: u16 = 2;

/// Ticks [`sparx_game`]'s Sparx is given to take every life: far more
/// than its laps of a [`SPARX_W`] x [`SPARX_H`] border need.
const MAX_GAME_TICKS: u32 = 10_000;

/// Ticks the tests that expect nothing to happen run.
const IDLE_TICKS: u32 = 20;

/// Inputs with neither press, which no state but [`GameState::Playing`]
/// acts on, each held for [`IDLE_TICKS`] by [`assert_waits`].
const OTHER_INPUTS: [InputState; 4] = [
    InputState {
        dir: None,
        fast: false,
        slow: false,
        confirm: false,
        pause: false,
    },
    InputState {
        dir: Some(Dir::Up),
        fast: true,
        slow: false,
        confirm: false,
        pause: false,
    },
    InputState {
        dir: Some(Dir::Left),
        fast: false,
        slow: true,
        confirm: false,
        pause: false,
    },
    InputState {
        dir: Some(Dir::Right),
        fast: false,
        slow: false,
        confirm: false,
        pause: false,
    },
];

const CONFIRM: InputState = InputState {
    dir: None,
    fast: false,
    slow: false,
    confirm: true,
    pause: false,
};

const PAUSE: InputState = InputState {
    dir: None,
    fast: false,
    slow: false,
    confirm: false,
    pause: true,
};

/// Holding up with the fast-draw button.
const DRAW_UP: InputState = InputState {
    dir: Some(Dir::Up),
    fast: true,
    slow: false,
    confirm: false,
    pause: false,
};

/// A game on a `W x H` board with every cell of [`CLAIMED_BEFORE`]
/// claimed, the Qix in cell `(0, 0)`, no Sparx and [`LOW_THRESHOLD`]: a
/// stix up from the marker completes the level.
fn level_game() -> Game {
    let mut board = Board::new(W, H);
    for &(x, y) in &CLAIMED_BEFORE {
        board.set_cell(x, y, Cell::Claimed(DrawKind::Fast));
    }
    let mut game = Game::with_board(board, vec![qix_at((0, 0))], Vec::new(), Rng::new(SEED));
    game.set_claim_threshold(ClaimThreshold::new(LOW_THRESHOLD));
    game
}

/// A game on a free `SPARX_W x SPARX_H` board with a still Qix in its
/// bottom-right cell and a Sparx one step right of the marker heading
/// for it: standing still, the marker loses a life every lap.
fn sparx_game() -> Game {
    let sparx = Sparx {
        pos: Point::new(SPARX_W / 2 + 1, SPARX_H),
        dir: Dir::Left,
        kind: SparxKind::Normal,
    };
    let board = Board::new(SPARX_W, SPARX_H);
    let qix = qix_at((SPARX_W - 1, SPARX_H - 1));
    Game::with_board(board, vec![qix], vec![sparx], Rng::new(SEED))
}

/// A session over `game`, confirmed past the title screen.
fn playing(game: Game) -> Session {
    let mut session = Session::new(game);
    session.tick(&CONFIRM);
    assert_eq!(session.state(), GameState::Playing);
    session
}

/// Ticks `session` `ticks` times with `input`.
fn hold(session: &mut Session, input: InputState, ticks: u32) {
    for _ in 0..ticks {
        session.tick(&input);
    }
}

/// A [`level_game`] played up to its completion.
fn completed_level() -> Session {
    let mut session = playing(level_game());
    hold(&mut session, DRAW_UP, u32::from(H) * MARKER_STEP_TICKS);
    session
}

/// A [`sparx_game`] played until the state leaves [`GameState::Playing`];
/// also returns how many deaths left it playing.
fn game_over() -> (Session, u32) {
    let mut session = playing(sparx_game());
    let mut deaths_playing = 0;
    for _ in 0..MAX_GAME_TICKS {
        session.tick(&InputState::default());
        if session.state() != GameState::Playing {
            break;
        }
        let died = session.game().events().iter();
        deaths_playing += died.filter(|e| matches!(e, Event::Death(_))).count() as u32;
    }
    (session, deaths_playing)
}

/// Ticks `session` with each of [`OTHER_INPUTS`] and asserts that neither
/// its state nor its game changes.
fn assert_waits(session: &mut Session) {
    let (state, game) = (session.state(), format!("{:?}", session.game()));
    for input in OTHER_INPUTS {
        hold(session, input, IDLE_TICKS);
        assert_eq!(session.state(), state, "{input:?}");
        assert_eq!(format!("{:?}", session.game()), game, "{input:?}");
    }
}

#[test]
fn a_new_session_is_on_the_title_screen() {
    let session = Session::new(level_game());

    assert_eq!(session.state(), GameState::Title);
    assert_eq!(session.game().ticks(), 0);
}

#[test]
fn the_title_screen_waits_for_confirm() {
    let mut session = Session::new(level_game());

    assert_waits(&mut session);
    hold(&mut session, PAUSE, IDLE_TICKS);

    assert_eq!(session.state(), GameState::Title);
    assert_eq!(session.game().ticks(), 0);
}

#[test]
fn confirm_on_the_title_screen_starts_play_from_the_next_tick() {
    let mut session = Session::new(level_game());

    session.tick(&CONFIRM);

    assert_eq!(session.state(), GameState::Playing);
    assert_eq!(session.game().ticks(), 0);
    session.tick(&InputState::default());
    assert_eq!(session.game().ticks(), 1);
}

#[test]
fn playing_ticks_the_game_and_ignores_confirm() {
    let mut session = playing(level_game());

    hold(&mut session, CONFIRM, IDLE_TICKS);

    assert_eq!(session.state(), GameState::Playing);
    assert_eq!(session.game().ticks(), u64::from(IDLE_TICKS));
}

#[test]
fn pause_while_playing_pauses_before_the_game_ticks() {
    let mut session = playing(level_game());
    session.tick(&InputState::default());

    session.tick(&PAUSE);

    assert_eq!(session.state(), GameState::Paused);
    assert_eq!(session.game().ticks(), 1);
}

#[test]
fn ticking_while_paused_changes_nothing() {
    let mut session = playing(level_game());
    hold(&mut session, DRAW_UP, MARKER_STEP_TICKS);
    assert!(session.game().stix().is_some(), "no stix to freeze");
    session.tick(&PAUSE);

    assert_waits(&mut session);
    hold(&mut session, CONFIRM, IDLE_TICKS);

    assert_eq!(session.state(), GameState::Paused);
    assert!(session.game().stix().is_some());
}

#[test]
fn pause_while_paused_resumes_play() {
    let mut session = playing(level_game());
    session.tick(&PAUSE);
    let ticks = session.game().ticks();

    session.tick(&PAUSE);

    assert_eq!(session.state(), GameState::Playing);
    assert_eq!(session.game().ticks(), ticks, "the resuming tick ticked");
    session.tick(&InputState::default());
    assert_eq!(session.game().ticks(), ticks + 1);
}

#[test]
fn a_completed_level_shows_its_claim_and_bonus() {
    let session = completed_level();

    assert_eq!(
        session.state(),
        GameState::LevelComplete {
            fraction: FRACTION_AFTER,
            bonus: PERCENT_ABOVE_LOW * BONUS_PER_PERCENT,
            split: false,
        }
    );
}

#[test]
fn a_completed_level_waits_for_confirm() {
    let mut session = completed_level();

    assert_waits(&mut session);
    hold(&mut session, PAUSE, IDLE_TICKS);

    assert!(matches!(session.state(), GameState::LevelComplete { .. }));
}

#[test]
fn confirm_after_a_completed_level_plays_the_next_level() {
    let mut session = completed_level();
    let score = session.game().score();

    session.tick(&CONFIRM);

    assert_eq!(session.state(), GameState::Playing);
    let game = session.game();
    assert_eq!(game.level(), FIRST_LEVEL + 1);
    assert_eq!(game.board().claimed_fraction(), 0.0);
    assert_eq!(game.score(), score);
    assert!(!game.is_level_complete());
}

#[test]
fn a_death_with_lives_left_keeps_playing() {
    let (_, deaths_playing) = game_over();

    assert_eq!(deaths_playing, STARTING_LIVES - 1);
}

#[test]
fn losing_the_last_life_ends_the_game() {
    let (session, _) = game_over();

    assert_eq!(session.state(), GameState::GameOver);
    assert_eq!(session.game().lives(), 0);
}

#[test]
fn game_over_waits_for_confirm() {
    let (mut session, _) = game_over();

    assert_waits(&mut session);
    hold(&mut session, PAUSE, IDLE_TICKS);

    assert_eq!(session.state(), GameState::GameOver);
}

#[test]
fn confirm_after_game_over_shows_the_title_with_a_new_game() {
    let (mut session, _) = game_over();

    session.tick(&CONFIRM);

    assert_eq!(session.state(), GameState::Title);
    let game = session.game();
    assert_eq!(game.lives(), STARTING_LIVES);
    assert_eq!(game.ticks(), 0);
    assert_eq!(game.level(), FIRST_LEVEL);
}
