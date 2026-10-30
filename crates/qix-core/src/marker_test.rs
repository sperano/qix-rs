use crate::tuning::{BOARD_H, BOARD_W};
use crate::{Board, Cell, Dir, DrawKind, Marker, MarkerState, Point};

/// Dimensions of the small board most tests use.
const W: u16 = 4;
const H: u16 = 3;

fn board() -> Board {
    Board::new(W, H)
}

#[test]
fn starts_on_the_border_at_the_bottom_edge_centre() {
    let board = Board::new(BOARD_W, BOARD_H);

    let marker = Marker::start(&board);

    assert_eq!(marker.pos, Point::new(BOARD_W / 2, BOARD_H));
    assert_eq!(marker.state, MarkerState::OnBorder);
    assert!(board.is_border(marker.pos));
}

#[test]
fn steps_along_a_border_edge() {
    let board = board();
    let mut marker = Marker::start(&board);

    assert!(marker.step_along_border(&board, Dir::Left));

    assert_eq!(marker.pos, Point::new(W / 2 - 1, H));
}

#[test]
fn refuses_a_step_into_free_space() {
    let board = board();
    let mut marker = Marker::start(&board);

    assert!(!marker.step_along_border(&board, Dir::Up));

    assert_eq!(marker.pos, Point::new(W / 2, H));
}

#[test]
fn refuses_a_step_off_the_board() {
    let board = board();
    let mut marker = Marker::start(&board);

    assert!(!marker.step_along_border(&board, Dir::Down));

    assert_eq!(marker.pos, Point::new(W / 2, H));
}

#[test]
fn turns_at_an_outer_corner() {
    let board = board();
    let mut marker = Marker {
        pos: Point::new(0, H),
        state: MarkerState::OnBorder,
    };

    assert!(!marker.step_along_border(&board, Dir::Left));
    assert!(marker.step_along_border(&board, Dir::Up));

    assert_eq!(marker.pos, Point::new(0, H - 1));
}

#[test]
fn turns_around_a_claimed_cell() {
    let mut board = board();
    board.set_cell(W - 1, H - 1, Cell::Claimed(DrawKind::Fast));
    let mut marker = Marker {
        pos: Point::new(W - 1, H),
        state: MarkerState::OnBorder,
    };

    assert!(!marker.step_along_border(&board, Dir::Right));
    assert!(marker.step_along_border(&board, Dir::Up));
    assert!(!marker.step_along_border(&board, Dir::Up));
    assert!(marker.step_along_border(&board, Dir::Right));

    assert_eq!(marker.pos, Point::new(W, H - 1));
}
