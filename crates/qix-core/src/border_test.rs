use std::collections::HashSet;

use crate::tuning::{BOARD_H, BOARD_W};
use crate::{Board, Cell, Dir, DrawKind, Point};

/// Dimensions of the small board most tests use.
const W: u16 = 4;
const H: u16 = 3;

/// The smallest number of border neighbours a border point can have: the
/// border never dead-ends.
const MIN_BORDER_NEIGHBOURS: usize = 2;

const FAST: Cell = Cell::Claimed(DrawKind::Fast);
const SLOW: Cell = Cell::Claimed(DrawKind::Slow);

fn board() -> Board {
    Board::new(W, H)
}

/// Every point of the board's `(W+1) x (H+1)` lattice.
fn lattice(board: &Board) -> impl Iterator<Item = Point> {
    let (w, h) = (board.width(), board.height());
    (0..=h).flat_map(move |y| (0..=w).map(move |x| Point::new(x, y)))
}

/// Every border edge, each once: as its left or top end and the step
/// `Right` or `Down` from there.
fn border_edges(board: &Board) -> HashSet<(Point, Dir)> {
    lattice(board)
        .flat_map(|point| {
            board
                .border_neighbours(point)
                .filter(|(dir, _)| matches!(dir, Dir::Right | Dir::Down))
                .map(move |(dir, _)| (point, dir))
        })
        .collect()
}

/// The unit edges of the four sides of a `w x h` board.
fn outer_edges(w: u16, h: u16) -> HashSet<(Point, Dir)> {
    let horizontal =
        (0..w).flat_map(|x| [Point::new(x, 0), Point::new(x, h)].map(|p| (p, Dir::Right)));
    let vertical =
        (0..h).flat_map(|y| [Point::new(0, y), Point::new(w, y)].map(|p| (p, Dir::Down)));
    horizontal.chain(vertical).collect()
}

fn neighbours(board: &Board, point: Point) -> Vec<(Dir, Point)> {
    board.border_neighbours(point).collect()
}

fn assert_no_dead_ends(board: &Board) {
    for point in lattice(board).filter(|&point| board.is_border(point)) {
        let count = board.border_neighbours(point).count();
        assert!(count >= MIN_BORDER_NEIGHBOURS, "{point:?} has {count}");
    }
}

#[test]
fn the_border_of_a_fresh_board_is_its_four_outer_edges() {
    assert_eq!(border_edges(&board()), outer_edges(W, H));
    let full = Board::new(BOARD_W, BOARD_H);
    assert_eq!(border_edges(&full), outer_edges(BOARD_W, BOARD_H));
}

#[test]
fn the_border_points_of_a_fresh_board_are_on_its_sides() {
    let board = board();
    for point in lattice(&board) {
        let on_side = point.x == 0 || point.x == W || point.y == 0 || point.y == H;
        assert_eq!(board.is_border(point), on_side, "{point:?}");
    }
}

#[test]
fn every_border_point_of_a_fresh_board_has_two_border_neighbours() {
    assert_no_dead_ends(&board());
    assert_no_dead_ends(&Board::new(BOARD_W, BOARD_H));
}

#[test]
fn a_corner_of_the_board_leads_along_both_sides() {
    let board = board();
    assert_eq!(
        neighbours(&board, Point::new(0, 0)),
        [
            (Dir::Down, Point::new(0, 1)),
            (Dir::Right, Point::new(1, 0))
        ]
    );
    assert_eq!(
        neighbours(&board, Point::new(W, H)),
        [
            (Dir::Up, Point::new(W, H - 1)),
            (Dir::Left, Point::new(W - 1, H))
        ]
    );
}

#[test]
fn each_neighbour_is_one_step_in_its_direction() {
    let mut board = board();
    board.set_cell(1, 1, FAST);
    for point in lattice(&board) {
        for (dir, next) in board.border_neighbours(point) {
            assert_eq!(dir.step(point), Some(next), "{point:?} {dir:?}");
        }
    }
}

#[test]
fn border_edges_lead_both_ways() {
    let mut board = board();
    board.set_cell(1, 1, FAST);
    board.set_cell(2, 2, SLOW);
    for point in lattice(&board) {
        for (dir, next) in board.border_neighbours(point) {
            let back = (dir.opposite(), point);
            assert!(
                neighbours(&board, next).contains(&back),
                "{point:?} {dir:?}"
            );
        }
    }
}

#[test]
fn a_claimed_inner_cell_adds_its_four_edges_to_the_border() {
    let mut board = board();
    board.set_cell(1, 1, SLOW);
    let mut expected = outer_edges(W, H);
    expected.extend([
        (Point::new(1, 1), Dir::Right),
        (Point::new(1, 2), Dir::Right),
        (Point::new(1, 1), Dir::Down),
        (Point::new(2, 1), Dir::Down),
    ]);
    assert_eq!(border_edges(&board), expected);
    assert_no_dead_ends(&board);
}

#[test]
fn claiming_along_a_side_moves_the_border_inwards() {
    let mut board = board();
    for x in 0..W {
        board.set_cell(x, 0, FAST);
    }
    let expected: HashSet<_> = outer_edges(W, H - 1)
        .into_iter()
        .map(|(point, dir)| (Point::new(point.x, point.y + 1), dir))
        .collect();
    assert_eq!(border_edges(&board), expected);
    assert!(!board.is_border(Point::new(1, 0)));
    assert!(!board.is_border(Point::new(0, 0)));
    assert_no_dead_ends(&board);
}

#[test]
fn diagonal_claimed_cells_meet_at_a_four_way_point() {
    let mut board = board();
    board.set_cell(0, 0, FAST);
    board.set_cell(1, 1, FAST);
    let point = Point::new(1, 1);
    assert!(board.is_border(point));
    let dirs: Vec<Dir> = board.border_neighbours(point).map(|(dir, _)| dir).collect();
    assert_eq!(dirs, Dir::ALL);
}

#[test]
fn a_fully_claimed_board_has_no_border() {
    let mut board = board();
    for y in 0..H {
        for x in 0..W {
            board.set_cell(x, y, FAST);
        }
    }
    assert!(lattice(&board).all(|point| !board.is_border(point)));
    assert!(border_edges(&board).is_empty());
}

#[test]
fn points_beyond_the_lattice_are_not_on_the_border() {
    let board = board();
    let beyond = [
        Point::new(W + 1, 0),
        Point::new(0, H + 1),
        Point::new(u16::MAX, u16::MAX),
    ];
    for point in beyond {
        assert!(!board.is_border(point), "{point:?}");
        assert!(neighbours(&board, point).is_empty(), "{point:?}");
    }
}

#[test]
fn edges_inside_a_fresh_board_are_free() {
    let board = board();
    let inner = Point::new(1, 1);

    for dir in Dir::ALL {
        assert!(board.is_free_edge(inner, dir), "{dir:?}");
    }
}

#[test]
fn border_edges_and_edges_off_the_lattice_are_not_free() {
    let board = board();
    let bottom = Point::new(W / 2, H);

    assert!(board.is_free_edge(bottom, Dir::Up));
    for dir in [Dir::Down, Dir::Left, Dir::Right] {
        assert!(!board.is_free_edge(bottom, dir), "{dir:?}");
    }
}

#[test]
fn an_edge_between_claimed_cells_is_not_free() {
    let mut board = board();
    board.set_cell(0, 0, FAST);
    board.set_cell(0, 1, SLOW);

    assert!(!board.is_free_edge(Point::new(1, 1), Dir::Left));
    assert!(board.is_free_edge(Point::new(1, 1), Dir::Right));
}

#[test]
fn is_border_edge_agrees_with_the_border_neighbours() {
    let mut board = board();
    board.set_cell(0, 0, FAST);
    board.set_cell(2, 1, SLOW);
    // One row beyond the lattice on each side, to cover edges leaving it.
    for y in 0..=H + 1 {
        for x in 0..=W + 1 {
            let point = Point::new(x, y);
            for dir in Dir::ALL {
                let listed = neighbours(&board, point).iter().any(|&(d, _)| d == dir);
                assert_eq!(
                    board.is_border_edge(point, dir),
                    listed,
                    "{point:?} {dir:?}"
                );
            }
        }
    }
}
