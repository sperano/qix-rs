use qix_core::{Board, Cell, DrawKind};

use crate::claim_fill::{CLAIM_FILL_TICKS, ClaimFill};
use crate::render::board::CellRun;

/// Dimensions of the small board the tests use.
const W: u16 = 6;
const H: u16 = 4;

const FAST: Cell = Cell::Claimed(DrawKind::Fast);
const SLOW: Cell = Cell::Claimed(DrawKind::Slow);

/// A `W x H` board with the cells `cells` claimed as given.
fn board_with(cells: &[(u16, u16, Cell)]) -> Board {
    let mut board = Board::new(W, H);
    for &(x, y, cell) in cells {
        board.set_cell(x, y, cell);
    }
    board
}

#[test]
fn a_new_animation_runs_nothing() {
    let fill = ClaimFill::new(&board_with(&[(0, 0, FAST)]));
    assert_eq!(fill.fresh(), None);
}

#[test]
fn an_unchanged_board_starts_nothing() {
    let board = board_with(&[(0, 0, FAST)]);
    let mut fill = ClaimFill::new(&board);
    fill.tick(&board);
    assert_eq!(fill.fresh(), None);
}

#[test]
fn a_claim_animates_only_the_cells_it_took() {
    let mut fill = ClaimFill::new(&board_with(&[(0, 0, FAST)]));
    fill.tick(&board_with(&[
        (0, 0, FAST),
        (1, 0, SLOW),
        (2, 0, SLOW),
        (4, 2, FAST),
    ]));
    let expected = [
        CellRun {
            row: 0,
            cols: 1..3,
            kind: DrawKind::Slow,
        },
        CellRun {
            row: 2,
            cols: 4..5,
            kind: DrawKind::Fast,
        },
    ];
    assert_eq!(fill.fresh(), Some((&expected[..], 0.0)));
}

#[test]
fn the_animation_moves_on_one_step_a_tick_and_ends_after_its_ticks() {
    let claimed = board_with(&[(3, 3, FAST)]);
    let mut fill = ClaimFill::new(&Board::new(W, H));
    fill.tick(&claimed);
    for done in 1..CLAIM_FILL_TICKS {
        fill.tick(&claimed);
        let (_, progress) = fill.fresh().expect("the animation runs");
        assert_eq!(progress, done as f32 / CLAIM_FILL_TICKS as f32);
    }
    fill.tick(&claimed);
    assert_eq!(fill.fresh(), None);
}

#[test]
fn a_new_claim_replaces_the_running_animation() {
    let first = board_with(&[(0, 0, FAST)]);
    let second = board_with(&[(0, 0, FAST), (5, 3, SLOW)]);
    let mut fill = ClaimFill::new(&Board::new(W, H));
    fill.tick(&first);
    fill.tick(&first);
    fill.tick(&second);
    let (runs, progress) = fill.fresh().expect("the animation runs");
    assert_eq!(progress, 0.0);
    assert_eq!(
        runs,
        [CellRun {
            row: 3,
            cols: 5..6,
            kind: DrawKind::Slow,
        }]
    );
}

#[test]
fn a_board_that_lost_claimed_cells_stops_the_animation() {
    let claimed = board_with(&[(0, 0, FAST)]);
    let mut fill = ClaimFill::new(&Board::new(W, H));
    fill.tick(&claimed);
    fill.tick(&board_with(&[(1, 1, FAST)]));
    assert_eq!(fill.fresh(), None);
}

#[test]
fn a_board_of_another_size_starts_nothing() {
    let mut fill = ClaimFill::new(&Board::new(W, H));
    let mut larger = Board::new(W + 1, H);
    larger.set_cell(0, 0, FAST);
    fill.tick(&larger);
    assert_eq!(fill.fresh(), None);
}

#[test]
fn a_claim_after_a_reset_is_compared_with_the_new_board() {
    let mut fill = ClaimFill::new(&board_with(&[(0, 0, FAST)]));
    fill.tick(&Board::new(W, H));
    fill.tick(&board_with(&[(2, 2, FAST)]));
    let (runs, _) = fill.fresh().expect("the animation runs");
    assert_eq!(
        runs,
        [CellRun {
            row: 2,
            cols: 2..3,
            kind: DrawKind::Fast,
        }]
    );
}
