use std::ops::Range;

use eframe::egui::{Rect, pos2};
use qix_core::{Board, Cell, DrawKind, Point, Stix};

use crate::render::board::{
    CellRun, border_segments, claimed_runs, diamond, square_capped, stix_runs,
};

/// Dimensions of the small board the tests use.
const W: u16 = 6;
const H: u16 = 4;

const FAST: Cell = Cell::Claimed(DrawKind::Fast);
const SLOW: Cell = Cell::Claimed(DrawKind::Slow);

fn run(row: u16, cols: Range<u16>, kind: DrawKind) -> CellRun {
    CellRun { row, cols, kind }
}

fn segment(from: (u16, u16), to: (u16, u16)) -> (Point, Point) {
    (Point::new(from.0, from.1), Point::new(to.0, to.1))
}

/// A `W x H` board with the cells of each `(rows, cols, cell)` block set to
/// `cell`.
fn board_with(cells: &[(Range<u16>, Range<u16>, Cell)]) -> Board {
    let mut board = Board::new(W, H);
    for (rows, cols, cell) in cells {
        for y in rows.clone() {
            for x in cols.clone() {
                board.set_cell(x, y, *cell);
            }
        }
    }
    board
}

#[test]
fn a_fresh_board_has_no_claimed_runs() {
    assert!(claimed_runs(&Board::new(W, H)).is_empty());
}

#[test]
fn neighbouring_cells_of_one_kind_merge_into_one_run() {
    let board = board_with(&[(1..2, 1..4, FAST)]);
    assert_eq!(claimed_runs(&board), [run(1, 1..4, DrawKind::Fast)]);
}

#[test]
fn a_change_of_kind_or_a_free_cell_ends_a_run() {
    let board = board_with(&[(0..1, 0..2, FAST), (0..1, 2..3, SLOW), (0..1, 4..6, SLOW)]);
    assert_eq!(
        claimed_runs(&board),
        [
            run(0, 0..2, DrawKind::Fast),
            run(0, 2..3, DrawKind::Slow),
            run(0, 4..6, DrawKind::Slow),
        ]
    );
}

#[test]
fn runs_never_span_two_rows() {
    let board = board_with(&[(2..4, 0..W, SLOW)]);
    assert_eq!(
        claimed_runs(&board),
        [run(2, 0..W, DrawKind::Slow), run(3, 0..W, DrawKind::Slow)]
    );
}

#[test]
fn a_run_covers_its_cells_in_playfield_units() {
    let rect = run(2, 1..4, DrawKind::Fast).rect();
    assert_eq!(rect, Rect::from_min_max(pos2(1.0, 2.0), pos2(4.0, 3.0)));
}

#[test]
fn the_border_of_a_fresh_board_is_its_four_sides() {
    assert_eq!(
        border_segments(&Board::new(W, H)),
        [
            segment((0, 0), (W, 0)),
            segment((0, H), (W, H)),
            segment((0, 0), (0, H)),
            segment((W, 0), (W, H)),
        ]
    );
}

#[test]
fn a_claimed_corner_moves_the_border_around_it() {
    let board = board_with(&[(0..2, 0..3, FAST)]);
    assert_eq!(
        border_segments(&board),
        [
            segment((3, 0), (W, 0)),
            segment((0, 2), (3, 2)),
            segment((0, H), (W, H)),
            segment((0, 2), (0, H)),
            segment((3, 0), (3, 2)),
            segment((W, 0), (W, H)),
        ]
    );
}

#[test]
fn claimed_cells_of_both_kinds_have_no_border_between_them() {
    let board = board_with(&[(0..H, 0..3, FAST), (0..H, 3..W, SLOW)]);
    assert!(border_segments(&board).is_empty());
}

#[test]
fn a_square_cap_extends_a_line_by_half_its_width_at_both_ends() {
    let horizontal = square_capped([pos2(10.0, 5.0), pos2(30.0, 5.0)], 2.0);
    assert_eq!(horizontal, [pos2(9.0, 5.0), pos2(31.0, 5.0)]);
    let vertical = square_capped([pos2(4.0, 20.0), pos2(4.0, 8.0)], 4.0);
    assert_eq!(vertical, [pos2(4.0, 22.0), pos2(4.0, 6.0)]);
}

#[test]
fn a_stix_without_segments_has_no_runs() {
    assert!(stix_runs(&Stix::new(Point::new(2, H))).is_empty());
}

#[test]
fn stix_segments_of_one_kind_form_one_polyline() {
    let mut stix = Stix::new(Point::new(2, H));
    stix.push(Point::new(2, 2), DrawKind::Fast);
    stix.push(Point::new(4, 2), DrawKind::Fast);
    let expected = [Point::new(2, H), Point::new(2, 2), Point::new(4, 2)];
    assert_eq!(stix_runs(&stix), [(DrawKind::Fast, &expected[..])]);
}

#[test]
fn a_change_of_kind_starts_a_polyline_at_the_last_point() {
    let mut stix = Stix::new(Point::new(2, H));
    stix.push(Point::new(2, 2), DrawKind::Slow);
    stix.push(Point::new(4, 2), DrawKind::Fast);
    stix.push(Point::new(4, 1), DrawKind::Fast);
    stix.push(Point::new(5, 1), DrawKind::Slow);
    let slow_start = [Point::new(2, H), Point::new(2, 2)];
    let fast = [Point::new(2, 2), Point::new(4, 2), Point::new(4, 1)];
    let slow_end = [Point::new(4, 1), Point::new(5, 1)];
    assert_eq!(
        stix_runs(&stix),
        [
            (DrawKind::Slow, &slow_start[..]),
            (DrawKind::Fast, &fast[..]),
            (DrawKind::Slow, &slow_end[..]),
        ]
    );
}

#[test]
fn the_diamond_has_its_corners_on_the_axes_clockwise_from_the_top() {
    assert_eq!(
        diamond(pos2(10.0, 20.0), 3.0),
        [
            pos2(10.0, 17.0),
            pos2(13.0, 20.0),
            pos2(10.0, 23.0),
            pos2(7.0, 20.0),
        ]
    );
}
