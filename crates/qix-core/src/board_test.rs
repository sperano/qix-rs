use crate::board::OUTSIDE;
use crate::tuning::{BOARD_H, BOARD_W};
use crate::{Board, Cell, DrawKind, Point};

/// Dimensions of the small board most tests use.
const W: u16 = 4;
const H: u16 = 3;

const FREE: Cell = Cell::Free;
const FAST: Cell = Cell::Claimed(DrawKind::Fast);
const SLOW: Cell = Cell::Claimed(DrawKind::Slow);

fn board() -> Board {
    Board::new(W, H)
}

#[test]
fn a_new_board_has_its_dimensions_and_only_free_cells() {
    let board = board();
    assert_eq!((board.width(), board.height()), (W, H));
    for y in 0..i32::from(H) {
        for x in 0..i32::from(W) {
            assert_eq!(board.cell(x, y), FREE, "({x}, {y})");
        }
    }
}

#[test]
fn every_cell_off_the_board_is_claimed() {
    let (w, h) = (i32::from(W), i32::from(H));
    let outside = [
        (-1, 0),
        (0, -1),
        (-1, -1),
        (w, 0),
        (0, h),
        (w, h),
        (w - 1, h),
        (w, h - 1),
        (i32::MIN, i32::MAX),
        (i32::MAX, i32::MIN),
    ];
    for (x, y) in outside {
        assert_eq!(board().cell(x, y), OUTSIDE, "({x}, {y})");
    }
    assert!(OUTSIDE.is_claimed());
}

#[test]
fn a_fresh_board_has_nothing_claimed() {
    assert_eq!(board().claimed_fraction(), 0.0);
    assert_eq!(Board::new(BOARD_W, BOARD_H).claimed_fraction(), 0.0);
}

#[test]
fn the_claimed_fraction_counts_both_draw_kinds() {
    let mut board = board();
    board.set_cell(0, 0, FAST);
    board.set_cell(W - 1, H - 1, SLOW);
    board.set_cell(1, 1, FAST);
    assert_eq!(board.claimed_fraction(), 3.0 / f32::from(W * H));
}

#[test]
fn a_fully_claimed_board_has_a_fraction_of_one() {
    let mut board = board();
    for y in 0..H {
        for x in 0..W {
            board.set_cell(x, y, SLOW);
        }
    }
    assert_eq!(board.claimed_fraction(), 1.0);
}

#[test]
fn set_cell_changes_only_that_cell() {
    let mut board = board();
    board.set_cell(2, 1, SLOW);
    assert_eq!(board.cell(2, 1), SLOW);
    assert_eq!(board.cell(1, 2), FREE);
    assert_eq!(board.cell(1, 1), FREE);
    assert_eq!(board.cell(2, 0), FREE);
}

#[test]
#[should_panic(expected = "off the")]
fn setting_a_cell_off_the_board_panics() {
    board().set_cell(W, 0, FAST);
}

#[test]
#[should_panic(expected = "empty board")]
fn a_board_without_cells_panics() {
    Board::new(0, H);
}

#[test]
fn cells_around_the_corners_have_one_cell_on_the_board() {
    let cases = [
        (Point::new(0, 0), [OUTSIDE, OUTSIDE, OUTSIDE, FREE]),
        (Point::new(W, 0), [OUTSIDE, OUTSIDE, FREE, OUTSIDE]),
        (Point::new(0, H), [OUTSIDE, FREE, OUTSIDE, OUTSIDE]),
        (Point::new(W, H), [FREE, OUTSIDE, OUTSIDE, OUTSIDE]),
    ];
    for (point, expected) in cases {
        assert_eq!(board().cells_around(point), expected, "{point:?}");
    }
}

#[test]
fn cells_around_the_edges_have_two_cells_on_the_board() {
    let cases = [
        (Point::new(1, 0), [OUTSIDE, OUTSIDE, FREE, FREE]),
        (Point::new(1, H), [FREE, FREE, OUTSIDE, OUTSIDE]),
        (Point::new(0, 1), [OUTSIDE, FREE, OUTSIDE, FREE]),
        (Point::new(W, 1), [FREE, OUTSIDE, FREE, OUTSIDE]),
    ];
    for (point, expected) in cases {
        assert_eq!(board().cells_around(point), expected, "{point:?}");
    }
}

#[test]
fn cells_around_an_inner_point_come_in_reading_order() {
    let mut board = board();
    board.set_cell(0, 0, FAST);
    board.set_cell(1, 1, SLOW);
    assert_eq!(
        board.cells_around(Point::new(1, 1)),
        [FAST, FREE, FREE, SLOW]
    );
}

#[test]
fn cells_around_a_point_beyond_the_lattice_are_all_outside() {
    let far = Point::new(u16::MAX, u16::MAX);
    assert_eq!(board().cells_around(far), [OUTSIDE; 4]);
}
