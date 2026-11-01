//! Completing a level through [`Game::tick`]: a claim reaching the claim
//! threshold, which leaves the multiplier as it is.

use super::{SEED, draw, drawing, hold, holding, penned};
use crate::qix_test::qix_at;
use crate::tuning::{
    BONUS_PER_PERCENT, CLAIM_THRESHOLD_DEFAULT, CLAIM_THRESHOLD_RANGE, FUSE_DELAY_TICKS,
    MARKER_STEP_TICKS, POINTS_PER_CELL, STARTING_MULTIPLIER,
};
use crate::{
    Board, Cell, ClaimThreshold, Dir, DrawKind, Event, Game, InputState, Point, Rng, Sparx,
    SparxKind,
};

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

/// The events of the step closing the stix before any end of the level:
/// the step itself and its claim.
const CLOSING_EVENTS: [Event; 2] = [
    Event::Draw(DrawKind::Fast),
    Event::Claim {
        cells: STIX_CLAIM_CELLS as usize,
        kind: DrawKind::Fast,
    },
];

/// The board's claimed fraction after the stix with every cell of
/// [`CLAIMED_BEFORE`] claimed: 18 cells of 24.
const FRACTION_AFTER: f32 = 0.75;

/// A threshold below [`FRACTION_AFTER`], and the whole percent points
/// between them.
const LOW_THRESHOLD: f32 = 0.5;
const PERCENT_ABOVE_LOW: u32 = 25;

/// Height of the board of [`the_completing_claim_puts_the_fuse_out`]:
/// tall enough for [`penned`] to keep its Qix in the left column, out of
/// reach of the stix while the fuse lights. Closing the stix claims more
/// than the lowest threshold of it.
const FUSE_H: u16 = 6;

/// Where the Sparx of [`game_with_sparx`] starts: on the top edge, left of
/// the stix, on a border the claim keeps.
const SPARX_START: Point = Point::new(1, 0);

/// A game on the `W x H` board with the first `claimed` cells of
/// [`CLAIMED_BEFORE`] claimed, the Qix in cell `(0, 0)` and `sparx`.
fn game_with(claimed: usize, sparx: Vec<Sparx>) -> Game {
    let mut board = Board::new(W, H);
    for &(x, y) in &CLAIMED_BEFORE[..claimed] {
        board.set_cell(x, y, Cell::Claimed(DrawKind::Fast));
    }
    Game::with_board(board, vec![qix_at((0, 0))], sparx, Rng::new(SEED))
}

/// A [`game_with`] no Sparx.
fn game_with_claimed(claimed: usize) -> Game {
    game_with(claimed, Vec::new())
}

/// A [`game_with`] every cell of [`CLAIMED_BEFORE`] claimed and a Sparx at
/// [`SPARX_START`] heading right along the top edge.
fn game_with_sparx() -> Game {
    let sparx = Sparx {
        pos: SPARX_START,
        dir: Dir::Right,
        kind: SparxKind::Normal,
    };
    game_with(CLAIMED_BEFORE.len(), vec![sparx])
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
        [
            CLOSING_EVENTS[0],
            CLOSING_EVENTS[1],
            Event::LevelComplete {
                fraction: FRACTION_AFTER,
                bonus: 0,
                split: false,
            },
        ]
    );
    let claim_points = STIX_CLAIM_CELLS * POINTS_PER_CELL;
    assert_eq!(
        game.score().points(),
        claim_points,
        "a bonus at the threshold"
    );
}

#[test]
fn a_threshold_completion_keeps_the_multiplier() {
    let mut game = game_with_claimed(CLAIMED_BEFORE.len());
    game.set_multiplier(STARTING_MULTIPLIER + 1);

    close_stix(&mut game);

    assert_eq!(
        game.events(),
        [
            CLOSING_EVENTS[0],
            CLOSING_EVENTS[1],
            Event::LevelComplete {
                fraction: FRACTION_AFTER,
                bonus: 0,
                split: false,
            },
        ]
    );
    assert_eq!(game.multiplier(), STARTING_MULTIPLIER + 1);
}

#[test]
fn a_claim_below_the_threshold_does_not_complete_the_level() {
    let mut game = game_with_claimed(CLAIMED_BEFORE.len() - 1);

    close_stix(&mut game);

    assert!(game.board().claimed_fraction() < CLAIM_THRESHOLD_DEFAULT);
    assert!(!game.is_level_complete());
    assert_eq!(game.events(), CLOSING_EVENTS);
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
    assert_eq!(
        game.events(),
        [
            CLOSING_EVENTS[0],
            CLOSING_EVENTS[1],
            Event::LevelComplete {
                fraction: FRACTION_AFTER,
                bonus,
                split: false,
            },
        ]
    );
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

#[test]
fn the_completing_claim_puts_the_fuse_out() {
    let (board, qix) = penned(W, FUSE_H, 0);
    let mut game = Game::with_board(board, vec![qix], Vec::new(), Rng::new(SEED));
    game.set_claim_threshold(ClaimThreshold::new(*CLAIM_THRESHOLD_RANGE.start()));
    draw(&mut game, Dir::Up, DrawKind::Fast, u32::from(FUSE_H) - 1);
    hold(&mut game, InputState::default(), FUSE_DELAY_TICKS);
    assert!(game.fuse().is_some(), "the fuse did not light");

    draw(&mut game, Dir::Up, DrawKind::Fast, 1);

    assert!(game.is_level_complete());
    assert_eq!(game.stix(), None);
    assert_eq!(game.fuse(), None);
}

#[test]
fn nothing_moves_after_the_completing_claim_in_its_tick() {
    let mut game = game_with_sparx();
    let mut below = game_with_sparx();
    below.set_claim_threshold(ClaimThreshold::new(*CLAIM_THRESHOLD_RANGE.end()));
    let input = drawing(Dir::Up, DrawKind::Fast);
    let ticks = u32::from(H) * MARKER_STEP_TICKS;
    hold(&mut game, input, ticks - 1);
    hold(&mut below, input, ticks - 1);
    assert!(!game.is_level_complete(), "the level completed early");
    let last = (game.qix().to_vec(), game.sparx().to_vec());

    game.tick(&input);
    below.tick(&input);

    assert!(game.is_level_complete());
    assert!(!below.is_level_complete());
    assert_eq!(
        game.events(),
        [
            CLOSING_EVENTS[0],
            CLOSING_EVENTS[1],
            Event::LevelComplete {
                fraction: FRACTION_AFTER,
                bonus: 0,
                split: false,
            },
        ]
    );
    assert_eq!((game.qix(), game.sparx()), (&last.0[..], &last.1[..]));
    assert_ne!(
        below.qix(),
        last.0,
        "the Qix stands still below the threshold"
    );
    assert_ne!(
        below.sparx(),
        last.1,
        "the Sparx stands still below the threshold"
    );
}
