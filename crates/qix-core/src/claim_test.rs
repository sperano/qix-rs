use crate::claim::{claim, flood_free};
use crate::tuning::{BOARD_H, BOARD_W};
use crate::{BitSet, Board, Cell, ClaimResult, Dir, DrawKind, Point, Stix};

/// Dimensions of the board the tests cut.
const W: u16 = 6;
const H: u16 = 5;

/// The column of the vertical cuts: cells left of it are `x < CUT_X`.
const CUT_X: u16 = 3;

/// The row of the horizontal cut: cells below it are `y >= CUT_Y`.
const CUT_Y: u16 = 2;

const FAST: DrawKind = DrawKind::Fast;
const SLOW: DrawKind = DrawKind::Slow;

/// A stix from `start` along `path`, every step drawn with `kind`.
fn drawn_with(start: Point, path: &[Dir], kind: DrawKind) -> Stix {
    let mut stix = Stix::new(start);
    for &dir in path {
        let next = dir.step(stix.head()).unwrap();
        stix.push(next, kind);
    }
    stix
}

/// `steps` steps towards `dir`.
fn run(dir: Dir, steps: u16) -> Vec<Dir> {
    vec![dir; usize::from(steps)]
}

/// A fast stix from the bottom edge at `CUT_X` straight up to the top.
fn straight_cut() -> Stix {
    drawn_with(Point::new(CUT_X, H), &run(Dir::Up, H), FAST)
}

/// The free cells of `board` in reading order.
fn free_cells(board: &Board) -> Vec<(u16, u16)> {
    cells(board)
        .filter(|&(x, y)| board.cell(i32::from(x), i32::from(y)) == Cell::Free)
        .collect()
}

/// Every cell of `board` in reading order.
fn cells(board: &Board) -> impl Iterator<Item = (u16, u16)> + use<> {
    let (width, height) = (board.width(), board.height());
    (0..height).flat_map(move |y| (0..width).map(move |x| (x, y)))
}

/// The cells `set` holds, as [`flood_free`] lays them out on `board`.
fn reached(board: &Board, set: &BitSet) -> Vec<(u16, u16)> {
    let width = usize::from(board.width());
    cells(board)
        .filter(|&(x, y)| set.contains(usize::from(y) * width + usize::from(x)))
        .collect()
}

/// The cells with `x < CUT_X` or not, depending on `left`.
fn side_of_cut(board: &Board, left: bool) -> Vec<(u16, u16)> {
    cells(board).filter(|&(x, _)| (x < CUT_X) == left).collect()
}

#[test]
fn a_straight_cut_stops_the_fill() {
    let board = Board::new(W, H);

    let set = flood_free(&board, &straight_cut(), (0, 2));

    assert_eq!(reached(&board, &set), side_of_cut(&board, true));
}

#[test]
fn a_straight_cut_claims_the_half_without_the_seed() {
    let mut board = Board::new(W, H);

    let count = claim(&mut board, &straight_cut(), (0, 2));

    assert_eq!(count, usize::from((W - CUT_X) * H));
    assert_eq!(free_cells(&board), side_of_cut(&board, true));
    for (x, y) in side_of_cut(&board, false) {
        assert_eq!(board.cell(i32::from(x), i32::from(y)), Cell::Claimed(FAST));
    }
}

#[test]
fn the_seed_side_of_a_cut_stays_free_whichever_side_it_is() {
    let mut board = Board::new(W, H);

    claim(&mut board, &straight_cut(), (W - 1, 0));

    assert_eq!(free_cells(&board), side_of_cut(&board, false));
}

#[test]
fn a_slow_stix_claims_slow() {
    let mut board = Board::new(W, H);
    let stix = drawn_with(Point::new(CUT_X, H), &run(Dir::Up, H), SLOW);

    claim(&mut board, &stix, (0, 0));

    assert_eq!(board.cell(i32::from(W - 1), 0), Cell::Claimed(SLOW));
}

/// The L-shaped cut: up from the bottom edge at x = 2 to y = 2, then right
/// to the right edge. It closes off the bottom right cells `x >= 2, y >= 2`.
fn l_cut() -> Stix {
    let mut path = run(Dir::Up, 3);
    path.extend(run(Dir::Right, W - 2));
    drawn_with(Point::new(2, H), &path, FAST)
}

/// The cells the L-shaped cut closes off.
fn inside_l(board: &Board) -> Vec<(u16, u16)> {
    cells(board).filter(|&(x, y)| x >= 2 && y >= 2).collect()
}

#[test]
fn an_l_shaped_cut_claims_its_corner_from_a_seed_outside() {
    let mut board = Board::new(W, H);

    let count = claim(&mut board, &l_cut(), (0, 0));

    let corner = inside_l(&board);
    assert_eq!(count, corner.len());
    let outside: Vec<_> = cells(&board)
        .filter(|cell| !corner.contains(cell))
        .collect();
    assert_eq!(free_cells(&board), outside);
}

#[test]
fn an_l_shaped_cut_keeps_its_corner_from_a_seed_inside() {
    let mut board = Board::new(W, H);

    claim(&mut board, &l_cut(), (W - 1, H - 1));

    assert_eq!(free_cells(&board), inside_l(&board));
}

#[test]
fn a_stix_touching_the_border_without_enclosing_claims_nothing() {
    let mut board = Board::new(W, H);
    let stix = drawn_with(Point::new(CUT_X, H), &run(Dir::Up, H - 1), FAST);

    let count = claim(&mut board, &stix, (0, H - 1));

    assert_eq!(count, 0);
    assert_eq!(board, Board::new(W, H));
}

#[test]
fn a_cut_ending_on_a_claimed_area_claims_only_free_cells() {
    let mut board = Board::new(W, H);
    let claimed_x = W - 1;
    for y in 0..H {
        board.set_cell(claimed_x, y, Cell::Claimed(FAST));
    }
    let stix = drawn_with(Point::new(0, CUT_Y), &run(Dir::Right, claimed_x), SLOW);

    let count = claim(&mut board, &stix, (0, 0));

    let below: Vec<_> = cells(&board)
        .filter(|&(x, y)| x < claimed_x && y >= CUT_Y)
        .collect();
    assert_eq!(count, below.len());
    for (x, y) in below {
        assert_eq!(board.cell(i32::from(x), i32::from(y)), Cell::Claimed(SLOW));
    }
    for y in 0..H {
        assert_eq!(
            board.cell(i32::from(claimed_x), i32::from(y)),
            Cell::Claimed(FAST)
        );
    }
    let above: Vec<_> = cells(&board)
        .filter(|&(x, y)| x < claimed_x && y < CUT_Y)
        .collect();
    assert_eq!(free_cells(&board), above);
}

#[test]
fn a_claimed_seed_reaches_nothing() {
    let mut board = Board::new(W, H);
    board.set_cell(0, 0, Cell::Claimed(FAST));

    let set = flood_free(&board, &straight_cut(), (0, 0));

    assert_eq!(set.count(), 0);
}

#[test]
fn a_seed_off_the_board_reaches_nothing() {
    let board = Board::new(W, H);

    let set = flood_free(&board, &straight_cut(), (W, 0));

    assert_eq!(set.count(), 0);
}

#[test]
fn the_fill_does_not_enter_claimed_cells() {
    let mut board = Board::new(W, H);
    board.set_cell(1, 1, Cell::Claimed(FAST));

    let set = flood_free(&board, &Stix::new(Point::new(0, H)), (0, 0));

    assert_eq!(set.count(), usize::from(W * H) - 1);
    assert!(!reached(&board, &set).contains(&(1, 1)));
}

#[test]
fn a_full_size_board_floods_without_recursion() {
    let board = Board::new(BOARD_W, BOARD_H);

    let set = flood_free(&board, &Stix::new(Point::new(0, BOARD_H)), (0, 0));

    assert_eq!(set.count(), usize::from(BOARD_W) * usize::from(BOARD_H));
}

#[test]
fn closing_a_cut_reports_its_cells_the_claimed_fraction_and_kind() {
    let mut board = Board::new(W, H);
    let stix = drawn_with(Point::new(CUT_X, H), &run(Dir::Up, H), SLOW);

    let result = board.close_stix(&stix, (0, 0));

    let cells = usize::from((W - CUT_X) * H);
    let expected = ClaimResult {
        cells,
        fraction: board.claimed_fraction(),
        kind: SLOW,
    };
    assert_eq!(result, Some(expected));
    assert_eq!(expected.fraction, cells as f32 / f32::from(W * H));
}

#[test]
fn closing_a_stix_that_encloses_nothing_reports_none() {
    let mut board = Board::new(W, H);
    let stix = drawn_with(Point::new(CUT_X, H), &run(Dir::Up, H - 1), FAST);

    assert_eq!(board.close_stix(&stix, (0, H - 1)), None);
    assert_eq!(board, Board::new(W, H));
}
