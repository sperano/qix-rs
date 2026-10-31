//! Closing a stix through [`Game::tick`]: the side holding the Qix stays
//! free.

use super::{SEED, draw, penned, walk};
use crate::tuning::{POINTS_PER_CELL, SLOW_DRAW_FACTOR, STARTING_MULTIPLIER};
use crate::{Board, Dir, DrawKind, Game, Rng};

/// Dimensions of the boards: the marker starts at `(W / 2, H)`, and the
/// stix runs straight along column `W / 2` from one edge to the other.
const W: u16 = 20;
const H: u16 = 10;

/// The free column holding the Qix when it is left of the stix, and when
/// it is right of it: [`penned`] keeps the Qix in it.
const LEFT_COLUMN: u16 = 3;
const RIGHT_COLUMN: u16 = 16;

/// The lattice column of the second stix of
/// [`claims_add_up_in_the_score`]: between the Qix's column and the first
/// stix, on the side the first claim left free.
const SECOND_STIX_X: u16 = 7;

/// A free `W x H` board but for the walls of `column`.
fn walled_board(column: u16) -> Board {
    penned(W, H, column).0
}

/// A game on the board walled around `column`, with a vertical Qix down
/// the middle of the column, too long to turn between its walls.
fn game_with_qix_in(column: u16) -> Game {
    let (board, qix) = penned(W, H, column);
    Game::with_board(board, vec![qix], Vec::new(), Rng::new(SEED))
}

/// Draws the stix up from the marker's start to the top edge.
fn close_from_bottom(game: &mut Game) {
    draw(game, Dir::Up, DrawKind::Fast, u32::from(H));
}

/// Walks the marker round the left and top edges to the top of its
/// column, then draws the stix down to the bottom edge.
fn close_from_top(game: &mut Game) {
    let half = u32::from(W / 2);
    walk(game, Dir::Left, half);
    walk(game, Dir::Up, u32::from(H));
    walk(game, Dir::Right, half);
    draw(game, Dir::Down, DrawKind::Fast, u32::from(H));
}

/// Every cell on the Qix's side of the stix, `x < W / 2` when `column` is
/// left of it, is as [`walled_board`] left it; every cell on the other
/// side is claimed.
fn assert_qix_side_kept(game: &Game, column: u16) {
    let qix_left = column < W / 2;
    let (cell_x, _) = game.qix()[0].cell();
    assert_eq!(cell_x, column, "the Qix left its column");
    let kept = walled_board(column);
    for y in 0..H {
        for x in 0..W {
            let (cx, cy) = (i32::from(x), i32::from(y));
            let cell = game.board().cell(cx, cy);
            if (x < W / 2) == qix_left {
                assert_eq!(cell, kept.cell(cx, cy), "({x}, {y}) on the Qix side");
            } else {
                assert!(cell.is_claimed(), "({x}, {y}) is free");
            }
        }
    }
}

#[test]
fn a_qix_left_of_a_stix_drawn_from_the_bottom_keeps_the_left() {
    let mut game = game_with_qix_in(LEFT_COLUMN);

    close_from_bottom(&mut game);

    assert_eq!(game.stix(), None);
    assert_qix_side_kept(&game, LEFT_COLUMN);
}

#[test]
fn a_qix_right_of_a_stix_drawn_from_the_bottom_keeps_the_right() {
    let mut game = game_with_qix_in(RIGHT_COLUMN);

    close_from_bottom(&mut game);

    assert_eq!(game.stix(), None);
    assert_qix_side_kept(&game, RIGHT_COLUMN);
}

#[test]
fn a_qix_left_of_a_stix_drawn_from_the_top_keeps_the_left() {
    let mut game = game_with_qix_in(LEFT_COLUMN);

    close_from_top(&mut game);

    assert_eq!(game.stix(), None);
    assert_qix_side_kept(&game, LEFT_COLUMN);
}

#[test]
fn a_qix_right_of_a_stix_drawn_from_the_top_keeps_the_right() {
    let mut game = game_with_qix_in(RIGHT_COLUMN);

    close_from_top(&mut game);

    assert_eq!(game.stix(), None);
    assert_qix_side_kept(&game, RIGHT_COLUMN);
}

/// The cells of `board` that are claimed.
fn claimed_count(board: &Board) -> u32 {
    let cells = (0..H).flat_map(|y| (0..W).map(move |x| (i32::from(x), i32::from(y))));
    let claimed = cells
        .filter(|&(x, y)| board.cell(x, y).is_claimed())
        .count();
    u32::try_from(claimed).unwrap()
}

/// The cells the stix up from the bottom claims, drawn `kind`, in a game
/// with the Qix left of it, and the points the game scores for them.
fn claim_from_bottom(kind: DrawKind) -> (u32, u32) {
    let mut game = game_with_qix_in(LEFT_COLUMN);
    let before = claimed_count(game.board());
    assert_eq!(game.score().points(), 0);

    draw(&mut game, Dir::Up, kind, u32::from(H));

    assert_eq!(game.stix(), None);
    let cells = claimed_count(game.board()) - before;
    (cells, game.score().points())
}

#[test]
fn a_fast_claim_scores_its_cells_at_the_starting_multiplier() {
    let (cells, points) = claim_from_bottom(DrawKind::Fast);

    assert!(cells > 0);
    assert_eq!(points, cells * POINTS_PER_CELL * STARTING_MULTIPLIER);
}

#[test]
fn a_slow_claim_scores_the_slow_draw_factor_times_more() {
    let (cells, points) = claim_from_bottom(DrawKind::Slow);

    assert!(cells > 0);
    assert_eq!(
        points,
        cells * POINTS_PER_CELL * STARTING_MULTIPLIER * SLOW_DRAW_FACTOR
    );
}

#[test]
fn claims_add_up_in_the_score() {
    let mut game = game_with_qix_in(LEFT_COLUMN);
    let before = claimed_count(game.board());
    close_from_bottom(&mut game);
    let first = claimed_count(game.board()) - before;

    walk(&mut game, Dir::Left, u32::from(W / 2 - SECOND_STIX_X));
    draw(&mut game, Dir::Down, DrawKind::Fast, u32::from(H));

    assert_eq!(game.stix(), None);
    let second = claimed_count(game.board()) - before - first;
    assert!(first > 0 && second > 0);
    assert_eq!(
        game.score().points(),
        (first + second) * POINTS_PER_CELL * STARTING_MULTIPLIER
    );
}
