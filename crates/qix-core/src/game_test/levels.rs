//! Completing a level through [`Game::tick`]: a claim reaching the claim
//! threshold.

use super::{SEED, draw, hold, holding};
use crate::qix_test::qix_at;
use crate::tuning::{BONUS_PER_PERCENT, CLAIM_THRESHOLD_DEFAULT, POINTS_PER_CELL};
use crate::{Board, Cell, ClaimThreshold, Dir, DrawKind, Event, Game, Rng};

/// Dimensions of the board: the stix runs up column `W / 2` from the
/// marker's start, leaving the Qix in cell `(0, 0)` on its left.
const W: u16 = 8;
const H: u16 = 3;

/// Cells left of the stix claimed before it, in rows `1..H`, from column
/// 0 on: clear of the stix's column, so its points stay off the border,
/// and of the top row, which keeps the Qix's area in one piece.
const CLAIMED_BEFORE: [(u16, u16); 6] = [(0, 1), (1, 1), (2, 1), (0, 2), (1, 2), (2, 2)];

/// The cells right of the stix, which closing it claims.
const STIX_CLAIM_CELLS: u32 = (W / 2) as u32 * H as u32;

/// The board's claimed fraction after the stix with every cell of
/// [`CLAIMED_BEFORE`] claimed: 18 cells of 24.
const FRACTION_AFTER: f32 = 0.75;

/// A threshold below [`FRACTION_AFTER`], and the whole percent points
/// between them.
const LOW_THRESHOLD: f32 = 0.5;
const PERCENT_ABOVE_LOW: u32 = 25;

/// A game on the `W x H` board with the first `claimed` cells of
/// [`CLAIMED_BEFORE`] claimed and the Qix in cell `(0, 0)`.
fn game_with_claimed(claimed: usize) -> Game {
    let mut board = Board::new(W, H);
    for &(x, y) in &CLAIMED_BEFORE[..claimed] {
        board.set_cell(x, y, Cell::Claimed(DrawKind::Fast));
    }
    Game::with_board(board, qix_at((0, 0)), Vec::new(), Rng::new(SEED))
}

/// Draws the stix up from the marker's start to the top edge.
fn close_stix(game: &mut Game) {
    draw(game, Dir::Up, DrawKind::Fast, u32::from(H));
    assert_eq!(game.stix(), None, "the stix did not close");
}

#[test]
fn a_new_game_has_the_default_threshold() {
    let game = game_with_claimed(0);

    assert_eq!(game.claim_threshold(), ClaimThreshold::default());
    assert!(!game.is_level_complete());
}

#[test]
fn a_claim_exactly_at_the_threshold_completes_the_level() {
    let mut game = game_with_claimed(CLAIMED_BEFORE.len());
    assert_eq!(CLAIM_THRESHOLD_DEFAULT, FRACTION_AFTER);

    close_stix(&mut game);

    assert_eq!(game.board().claimed_fraction(), FRACTION_AFTER);
    assert!(game.is_level_complete());
    assert_eq!(
        game.events(),
        [Event::LevelComplete {
            fraction: FRACTION_AFTER
        }]
    );
    let claim_points = STIX_CLAIM_CELLS * POINTS_PER_CELL;
    assert_eq!(
        game.score().points(),
        claim_points,
        "a bonus at the threshold"
    );
}

#[test]
fn a_claim_below_the_threshold_does_not_complete_the_level() {
    let mut game = game_with_claimed(CLAIMED_BEFORE.len() - 1);

    close_stix(&mut game);

    assert!(game.board().claimed_fraction() < CLAIM_THRESHOLD_DEFAULT);
    assert!(!game.is_level_complete());
    assert_eq!(game.events(), []);
    let ticks = game.ticks();
    game.tick(&holding(Dir::Left));
    assert_eq!(game.ticks(), ticks + 1, "the game stopped ticking");
}

#[test]
fn the_bonus_scores_each_whole_percent_above_the_threshold() {
    let mut game = game_with_claimed(CLAIMED_BEFORE.len());
    game.set_claim_threshold(ClaimThreshold::new(LOW_THRESHOLD));

    close_stix(&mut game);

    assert!(game.is_level_complete());
    let claim_points = STIX_CLAIM_CELLS * POINTS_PER_CELL;
    let bonus = PERCENT_ABOVE_LOW * BONUS_PER_PERCENT;
    assert_eq!(game.score().points(), claim_points + bonus);
}

#[test]
fn a_completed_level_no_longer_ticks() {
    let mut game = game_with_claimed(CLAIMED_BEFORE.len());
    close_stix(&mut game);
    let (ticks, marker, score) = (game.ticks(), game.marker(), game.score());

    hold(&mut game, holding(Dir::Left), u32::from(W));

    assert_eq!(game.ticks(), ticks);
    assert_eq!(game.marker(), marker);
    assert_eq!(game.score(), score);
    assert_eq!(game.events(), []);
}
