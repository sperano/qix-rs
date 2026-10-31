//! Starting a level from the level table: [`Game::new`] and
//! [`Game::start_level`].

use super::deaths::{die_by_fuse, stand};
use super::{H, SEED, W, draw, new_game};
use crate::qix_test::qix_at;
use crate::tuning::{
    BOARD_H, BOARD_W, CLAIM_THRESHOLD_RANGE, DIFFICULTY_RAMP_LEVELS, FIRST_LEVEL, POINTS_PER_CELL,
    STARTING_LIVES, STARTING_MULTIPLIER, TWO_QIX_FROM_LEVEL,
};
use crate::{
    Board, ClaimThreshold, Dir, DrawKind, Game, InputState, LevelConfig, MarkerState, Point, Rng,
    Sparx, SparxMeter, Vec2,
};

/// Levels the set-up tests start: the first, the first with two Qix and
/// the end of the difficulty ramp.
const LEVELS: [u32; 3] = [
    FIRST_LEVEL,
    TWO_QIX_FROM_LEVEL,
    FIRST_LEVEL + DIFFICULTY_RAMP_LEVELS,
];

/// The level [`a_played_game`] goes on to.
const NEXT_LEVEL: u32 = FIRST_LEVEL + 1;

/// Rounding slack on where the Qix start.
const EPSILON: f32 = 1e-4;

/// Ticks the two-Qix game runs to see both Qix move.
const RUN_TICKS: u32 = 10;

/// `game` is set up as [`LevelConfig::for_level`] says for `level`.
fn assert_set_up_for(game: &Game, level: u32) {
    let config = LevelConfig::for_level(level);
    assert_eq!(game.level(), level);
    assert_eq!(game.qix().len(), config.qix_count, "level {level}");
    for qix in game.qix() {
        assert_eq!(qix.speed(), &config.qix_speed, "level {level}");
    }
    let sparx = Sparx::spawn_level(game.board(), config.sparx_count);
    assert_eq!(game.sparx(), sparx, "level {level}");
    assert_eq!(
        game.sparx_meter(),
        SparxMeter::new(config.meter_ticks),
        "level {level}"
    );
}

/// A full-size game with its Qix in the top-left cell and no Sparx that
/// lost a life to the fuse, then claimed the cell left of the marker's
/// start with a fast stix.
fn a_played_game() -> Game {
    let board = Board::new(BOARD_W, BOARD_H);
    let mut game = Game::with_board(board, vec![qix_at((0, 0))], Vec::new(), Rng::new(SEED));
    die_by_fuse(&mut game);
    let grace = game.grace_ticks();
    stand(&mut game, grace);
    for dir in [Dir::Up, Dir::Left, Dir::Down] {
        draw(&mut game, dir, DrawKind::Fast, 1);
    }
    assert_eq!(game.score().points(), POINTS_PER_CELL, "nothing claimed");
    game
}

/// A full-size game with its Qix in the top-left cell, no Sparx and the
/// lowest claim threshold, which a stix up the middle completes by
/// claiming the right half.
fn a_completed_level() -> Game {
    let board = Board::new(BOARD_W, BOARD_H);
    let mut game = Game::with_board(board, vec![qix_at((0, 0))], Vec::new(), Rng::new(SEED));
    game.set_claim_threshold(ClaimThreshold::new(*CLAIM_THRESHOLD_RANGE.start()));
    draw(&mut game, Dir::Up, DrawKind::Fast, u32::from(BOARD_H));
    assert!(game.is_level_complete(), "the level did not complete");
    game
}

#[test]
fn a_new_game_starts_the_first_level() {
    assert_set_up_for(&new_game(), FIRST_LEVEL);
}

#[test]
fn a_level_is_set_up_from_its_row_of_the_table() {
    for level in LEVELS {
        let mut game = new_game();

        game.start_level(level);

        assert_set_up_for(&game, level);
    }
}

#[test]
fn a_level_below_the_first_starts_the_first() {
    let mut game = new_game();

    game.start_level(FIRST_LEVEL - 1);

    assert_set_up_for(&game, FIRST_LEVEL);
}

/// The Qix of `game` are centred at `xs`, as fractions of the board's
/// width, in order, on its middle row.
fn assert_qix_centred_at(game: &Game, xs: &[f32]) {
    assert_eq!(game.qix().len(), xs.len());
    let (width, height) = (f32::from(BOARD_W), f32::from(BOARD_H));
    for (qix, &x) in game.qix().iter().zip(xs) {
        let mid = qix.a().midpoint(qix.b());
        let expected = Vec2::new(width * x, height * 0.5);
        assert!(
            (mid - expected).length() < EPSILON,
            "{mid:?}, not {expected:?}"
        );
    }
}

#[test]
fn a_single_qix_starts_at_the_centre() {
    assert_qix_centred_at(&new_game(), &[0.5]);
}

#[test]
fn two_qix_start_a_third_of_the_way_in_from_either_side() {
    let mut game = new_game();

    game.start_level(TWO_QIX_FROM_LEVEL);

    assert_qix_centred_at(&game, &[1.0 / 3.0, 2.0 / 3.0]);
}

#[test]
fn both_qix_move() {
    let mut game = new_game();
    game.start_level(TWO_QIX_FROM_LEVEL);
    let start = game.qix().to_vec();

    for _ in 0..RUN_TICKS {
        game.tick(&InputState::default());
    }

    for (qix, start) in game.qix().iter().zip(&start) {
        assert_ne!((qix.a(), qix.b()), (start.a(), start.b()));
    }
}

#[test]
fn starting_a_level_keeps_the_score_and_the_lives() {
    let mut game = a_played_game();
    let (score, lives) = (game.score(), game.lives());
    assert!(lives < STARTING_LIVES, "no life lost");

    game.start_level(NEXT_LEVEL);

    assert_eq!((game.score(), game.lives()), (score, lives));
}

#[test]
fn starting_a_level_frees_the_board_and_puts_the_marker_back() {
    let mut game = a_played_game();
    assert_ne!(game.board(), &Board::new(BOARD_W, BOARD_H));

    game.start_level(NEXT_LEVEL);

    assert_eq!(game.board(), &Board::new(BOARD_W, BOARD_H));
    assert_eq!(game.marker().pos, Point::new(BOARD_W / 2, BOARD_H));
    assert_eq!(game.marker().state, MarkerState::OnBorder);
    assert_set_up_for(&game, NEXT_LEVEL);
}

#[test]
fn starting_a_level_drops_the_stix() {
    let mut game = new_game();
    draw(&mut game, Dir::Up, DrawKind::Fast, 1);
    assert!(game.stix().is_some());

    game.start_level(NEXT_LEVEL);

    assert_eq!(game.stix(), None);
    assert_eq!(game.fuse(), None);
}

#[test]
fn starting_a_level_ends_the_grace_period() {
    let mut game = Game::with_board(
        Board::new(BOARD_W, BOARD_H),
        vec![qix_at((0, 0))],
        Vec::new(),
        Rng::new(SEED),
    );
    die_by_fuse(&mut game);
    assert!(game.grace_ticks() > 0);

    game.start_level(NEXT_LEVEL);

    assert_eq!(game.grace_ticks(), 0);
}

#[test]
fn starting_a_level_after_completing_one_plays_on() {
    let mut game = a_completed_level();
    let threshold = game.claim_threshold();

    game.start_level(NEXT_LEVEL);
    let ticks = game.ticks();
    game.tick(&InputState::default());

    assert!(!game.is_level_complete());
    assert_eq!(game.ticks(), ticks + 1);
    assert_eq!(game.claim_threshold(), threshold);
}

#[test]
fn starting_a_level_keeps_the_tick_count() {
    let mut game = new_game();
    game.tick(&InputState::default());

    game.start_level(NEXT_LEVEL);

    assert_eq!(game.ticks(), 1);
}

#[test]
fn a_restart_starts_a_new_game() {
    let mut game = a_played_game();
    game.start_level(NEXT_LEVEL);
    game.set_multiplier(STARTING_MULTIPLIER + 1);
    game.set_claim_threshold(ClaimThreshold::new(*CLAIM_THRESHOLD_RANGE.start()));

    game.restart();

    assert_set_up_for(&game, FIRST_LEVEL);
    assert_eq!(game.ticks(), 0);
    assert_eq!(game.lives(), STARTING_LIVES);
    assert_eq!(game.score().points(), 0);
    assert_eq!(game.multiplier(), STARTING_MULTIPLIER);
    assert_eq!(game.claim_threshold(), ClaimThreshold::default());
}

#[test]
fn a_restart_frees_the_board() {
    let mut game = a_played_game();
    assert!(game.board().claimed_fraction() > 0.0, "nothing claimed");

    game.restart();

    assert_eq!(game.board(), &Board::new(BOARD_W, BOARD_H));
}

#[test]
fn a_restart_keeps_the_size_of_the_board() {
    let mut game = Game::with_board(
        Board::new(W, H),
        vec![qix_at((0, 0))],
        Vec::new(),
        Rng::new(SEED),
    );

    game.restart();

    assert_eq!((game.board().width(), game.board().height()), (W, H));
}
