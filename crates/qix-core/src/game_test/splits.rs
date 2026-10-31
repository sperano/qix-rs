//! Ending a level by splitting the two Qix apart, and the multiplier it
//! raises.

use super::{SEED, draw, holding, pen};
use crate::qix_test::qix_at;
use crate::tuning::{
    BOARD_H, BOARD_W, CLAIM_THRESHOLD_RANGE, FIRST_LEVEL, MAX_MULTIPLIER, POINTS_PER_CELL,
    STARTING_MULTIPLIER,
};
use crate::{Board, Cell, ClaimThreshold, Dir, DrawKind, Event, Game, Qix, Rng};

/// The level a split game goes on to.
const NEXT_LEVEL: u32 = FIRST_LEVEL + 1;

/// How far in from a side of the full board [`sides_game`] pens a Qix.
const PEN_COLUMN: u16 = 1;

/// The column between [`PEN_COLUMN`] and the middle where
/// [`a_claim_keeping_both_qix_together_is_no_split`] pens its second Qix.
const SECOND_PEN_COLUMN: u16 = 3;

/// Dimensions of the board of [`a_split_claim`]: the stix runs up
/// column `POCKET_W / 2` from the marker's start.
const POCKET_W: u16 = 8;
const POCKET_H: u16 = 4;

/// The columns [`a_split_claim`] claims before the stix: the first walls
/// a Qix into column 0, the others narrow the area of the second Qix.
const POCKET_CLAIMED_COLUMNS: [u16; 3] = [1, 6, 7];

/// The cells the stix of [`a_split_claim`] claims: columns 2 and 3, left
/// of it, with no Qix.
const POCKET_STIX_CELLS: u32 = 2 * POCKET_H as u32;

/// The claimed fraction after the stix of [`a_split_claim`]: 20 cells of
/// 32, above the lowest threshold.
const POCKET_FRACTION: f32 = 0.625;

/// A full-size game with no Sparx and a Qix [`pen`]ned in each of
/// `columns`.
fn two_qix_game(columns: [u16; 2]) -> Game {
    let mut board = Board::new(BOARD_W, BOARD_H);
    let qix = columns.map(|column| pen(&mut board, column)).to_vec();
    Game::with_board(board, qix, Vec::new(), Rng::new(SEED))
}

/// A [`two_qix_game`] with a Qix near either side, which a stix up the
/// middle splits.
fn sides_game() -> Game {
    two_qix_game([PEN_COLUMN, BOARD_W - 1 - PEN_COLUMN])
}

/// Draws the stix up the middle of the full board, from the marker's start
/// to the top edge, and checks that it closed without a death.
fn close_up_the_middle(game: &mut Game) {
    let lives = game.lives();
    draw(game, Dir::Up, DrawKind::Fast, u32::from(BOARD_H));
    assert_eq!(game.lives(), lives, "the stix was hit");
    assert_eq!(game.stix(), None, "the stix did not close");
}

/// A `POCKET_W x POCKET_H` game at the lowest threshold whose stix up the
/// middle claims [`POCKET_STIX_CELLS`] and leaves a Qix in column 0, walled
/// off by claimed column 1, and another in column 5, right of the stix.
fn a_split_claim() -> Game {
    let mut board = Board::new(POCKET_W, POCKET_H);
    for x in POCKET_CLAIMED_COLUMNS {
        for y in 0..POCKET_H {
            board.set_cell(x, y, Cell::Claimed(DrawKind::Fast));
        }
    }
    let qix: Vec<Qix> = [(0, 0), (5, 0)].map(qix_at).to_vec();
    let mut game = Game::with_board(board, qix, Vec::new(), Rng::new(SEED));
    game.set_claim_threshold(ClaimThreshold::new(*CLAIM_THRESHOLD_RANGE.start()));
    draw(&mut game, Dir::Up, DrawKind::Fast, u32::from(POCKET_H));
    assert_eq!(game.stix(), None, "the stix did not close");
    game
}

#[test]
fn a_stix_splitting_the_qix_ends_the_level() {
    let mut game = sides_game();
    let board = game.board().clone();

    close_up_the_middle(&mut game);

    assert!(game.is_level_complete());
    assert_eq!(game.board(), &board, "the stix claimed cells");
    assert_eq!(
        game.events(),
        [Event::LevelComplete {
            fraction: game.board().claimed_fraction(),
            split: true,
        }]
    );
    assert_eq!(game.score().points(), 0, "nothing claimed, no bonus");
}

#[test]
fn a_split_level_no_longer_ticks() {
    let mut game = sides_game();
    close_up_the_middle(&mut game);
    let ticks = game.ticks();

    game.tick(&holding(Dir::Left));

    assert_eq!(game.ticks(), ticks);
    assert_eq!(game.events(), []);
}

#[test]
fn a_split_raises_the_multiplier_by_one() {
    let mut game = sides_game();
    assert_eq!(game.multiplier(), STARTING_MULTIPLIER);

    close_up_the_middle(&mut game);

    assert_eq!(game.multiplier(), STARTING_MULTIPLIER + 1);
}

#[test]
fn a_split_raises_a_raised_multiplier_by_one() {
    let mut game = sides_game();
    game.set_multiplier(STARTING_MULTIPLIER + 1);

    close_up_the_middle(&mut game);

    assert_eq!(game.multiplier(), STARTING_MULTIPLIER + 2);
}

#[test]
fn a_claim_keeping_both_qix_together_is_no_split() {
    let mut game = two_qix_game([PEN_COLUMN, SECOND_PEN_COLUMN]);

    close_up_the_middle(&mut game);

    assert!(game.board().claimed_fraction() > 0.0, "nothing claimed");
    assert!(!game.is_level_complete());
    assert_eq!(game.events(), []);
    assert_eq!(game.multiplier(), STARTING_MULTIPLIER);
}

#[test]
fn a_split_claim_scores_at_the_old_multiplier_without_a_bonus() {
    let game = a_split_claim();

    assert_eq!(game.board().claimed_fraction(), POCKET_FRACTION);
    assert_eq!(
        game.events(),
        [Event::LevelComplete {
            fraction: POCKET_FRACTION,
            split: true,
        }]
    );
    let claim_points = POCKET_STIX_CELLS * POINTS_PER_CELL * STARTING_MULTIPLIER;
    assert_eq!(game.score().points(), claim_points, "a bonus on a split");
    assert_eq!(game.multiplier(), STARTING_MULTIPLIER + 1);
}

#[test]
fn the_raised_multiplier_scores_the_next_level() {
    let mut game = sides_game();
    close_up_the_middle(&mut game);

    game.start_level(NEXT_LEVEL);
    for dir in [Dir::Up, Dir::Left, Dir::Down] {
        draw(&mut game, dir, DrawKind::Fast, 1);
    }

    assert_eq!(game.multiplier(), STARTING_MULTIPLIER + 1);
    let points = POINTS_PER_CELL * (STARTING_MULTIPLIER + 1);
    assert_eq!(game.score().points(), points, "one cell claimed");
}

#[test]
fn a_split_at_the_highest_multiplier_keeps_it() {
    let mut game = sides_game();
    game.set_multiplier(MAX_MULTIPLIER);

    close_up_the_middle(&mut game);

    assert!(game.is_level_complete());
    assert_eq!(game.multiplier(), MAX_MULTIPLIER);
}

#[test]
fn the_multiplier_is_set_within_its_range() {
    let mut game = sides_game();

    game.set_multiplier(MAX_MULTIPLIER + 1);
    assert_eq!(game.multiplier(), MAX_MULTIPLIER);
    game.set_multiplier(STARTING_MULTIPLIER - 1);
    assert_eq!(game.multiplier(), STARTING_MULTIPLIER);
}
