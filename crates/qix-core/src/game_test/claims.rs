//! Closing a stix through [`Game::tick`]: the side holding the Qix stays
//! free.

use super::{SEED, draw, walk};
use crate::{Board, Cell, Dir, DrawKind, Game, Qix, Rng, Vec2};

/// Dimensions of the boards: the marker starts at `(W / 2, H)`, and the
/// stix runs straight along column `W / 2` from one edge to the other.
const W: u16 = 20;
const H: u16 = 10;

/// The free column holding the Qix when it is left of the stix, and when
/// it is right of it: claimed walls flank each from the second row to the
/// second last, so the Qix stays in it however it moves.
const LEFT_COLUMN: u16 = 3;
const RIGHT_COLUMN: u16 = 16;

const WALL: Cell = Cell::Claimed(DrawKind::Fast);

/// The cells beside `column` from the second row to the second last.
fn is_wall(column: u16, x: u16, y: u16) -> bool {
    (x + 1 == column || x == column + 1) && (1..H - 1).contains(&y)
}

/// A free `W x H` board but for the walls of `column`.
fn walled_board(column: u16) -> Board {
    let mut board = Board::new(W, H);
    for y in 0..H {
        for x in 0..W {
            if is_wall(column, x, y) {
                board.set_cell(x, y, WALL);
            }
        }
    }
    board
}

/// A game on the board walled around `column`, with a vertical Qix down
/// the middle of the column, too long to turn between its walls.
fn game_with_qix_in(column: u16) -> Game {
    let x = f32::from(column) + 0.5;
    let (top, bottom) = (0.5, f32::from(H) - 1.5);
    let qix = Qix::new(
        Vec2::new(x, top),
        Vec2::new(x, bottom),
        Vec2::default(),
        0.0,
    );
    Game::with_board(walled_board(column), qix, Vec::new(), Rng::new(SEED))
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
    let (cell_x, _) = game.qix().cell();
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
