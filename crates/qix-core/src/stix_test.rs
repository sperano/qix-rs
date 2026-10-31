use crate::stix::{legal_moves, legal_step};
use crate::{Board, Cell, Dir, DrawKind, Marker, MarkerState, Point, Stix};

/// Dimensions of the board the tests draw on.
const W: u16 = 6;
const H: u16 = 5;

/// Where the stixes in these tests leave the border: the bottom edge.
const START: Point = Point::new(3, H);

const FAST: DrawKind = DrawKind::Fast;
const SLOW: DrawKind = DrawKind::Slow;

fn board() -> Board {
    Board::new(W, H)
}

/// A stix from `start` along `path`, every step drawn with `kind`.
fn drawn_with(start: Point, path: &[Dir], kind: DrawKind) -> Stix {
    let mut stix = Stix::new(start);
    for &dir in path {
        let next = dir.step(stix.head()).unwrap();
        stix.push(next, kind);
    }
    stix
}

/// A fast stix from [`START`] along `path`.
fn drawn(path: &[Dir]) -> Stix {
    drawn_with(START, path, FAST)
}

/// The marker drawing `stix`, at its head.
fn marker_on(stix: &Stix) -> Marker {
    Marker {
        pos: stix.head(),
        state: MarkerState::Drawing,
    }
}

fn step(stix: &Stix, board: &Board, dir: Dir) -> Option<Point> {
    legal_step(&marker_on(stix), stix, board, dir)
}

#[test]
fn a_new_stix_is_its_start() {
    let stix = Stix::new(START);

    assert_eq!(stix.start(), START);
    assert_eq!(stix.head(), START);
    assert_eq!(stix.points(), [START]);
    assert!(stix.kinds().is_empty());
    assert!(stix.contains_point(START));
}

#[test]
fn straight_steps_of_one_kind_lengthen_one_segment() {
    let stix = drawn(&[Dir::Up, Dir::Up, Dir::Up]);

    assert_eq!(stix.points(), [START, Point::new(3, H - 3)]);
    assert_eq!(stix.kinds(), [FAST]);
}

#[test]
fn a_turn_starts_a_segment() {
    let stix = drawn(&[Dir::Up, Dir::Up, Dir::Left]);

    let corner = Point::new(3, H - 2);
    assert_eq!(stix.points(), [START, corner, Point::new(2, H - 2)]);
    assert_eq!(stix.kinds(), [FAST, FAST]);
    assert_eq!(stix.head(), Point::new(2, H - 2));
}

#[test]
fn a_change_of_button_starts_a_segment() {
    let mut stix = drawn(&[Dir::Up]);

    stix.push(Point::new(3, H - 2), SLOW);

    let points = [START, Point::new(3, H - 1), Point::new(3, H - 2)];
    assert_eq!(stix.points(), points);
    assert_eq!(stix.kinds(), [FAST, SLOW]);
}

#[test]
#[should_panic(expected = "not straight")]
fn a_diagonal_step_panics() {
    let mut stix = Stix::new(START);

    stix.push(Point::new(2, H - 1), FAST);
}

#[test]
fn contains_every_point_along_its_segments() {
    let stix = drawn(&[Dir::Up, Dir::Up, Dir::Left]);

    for point in [START, Point::new(3, H - 1), Point::new(2, H - 2)] {
        assert!(stix.contains_point(point), "{point:?}");
    }
    assert!(!stix.contains_point(Point::new(2, H - 1)));
    assert!(!stix.contains_point(Point::new(3, H - 3)));
}

#[test]
fn contains_the_unit_edges_it_covers_either_way_round() {
    let stix = drawn(&[Dir::Up, Dir::Up, Dir::Left]);
    let (a, b) = (Point::new(3, H - 1), Point::new(3, H - 2));
    let left = Point::new(2, H - 2);

    assert!(stix.contains_segment(a, b));
    assert!(stix.contains_segment(b, a));
    assert!(stix.contains_segment(b, left));
    assert!(!stix.contains_segment(a, Point::new(2, H - 1)));
    assert!(!stix.contains_segment(left, Point::new(2, H - 1)));
    assert!(!stix.contains_segment(a, a));
}

#[test]
fn a_stix_drawn_slow_throughout_claims_slow() {
    let stix = drawn_with(START, &[Dir::Up, Dir::Left], SLOW);

    assert_eq!(stix.claim_kind(), SLOW);
}

#[test]
fn a_single_fast_segment_makes_the_claim_fast() {
    let mut stix = drawn_with(START, &[Dir::Up, Dir::Up], SLOW);
    stix.push(Point::new(2, H - 2), FAST);
    stix.push(Point::new(1, H - 2), SLOW);

    assert_eq!(stix.claim_kind(), FAST);
}

#[test]
fn a_stix_without_segments_claims_fast() {
    assert_eq!(Stix::new(START).claim_kind(), FAST);
}

#[test]
fn the_first_step_leaves_the_border_into_free_space_only() {
    let stix = Stix::new(START);

    assert_eq!(legal_moves(&marker_on(&stix), &stix, &board()), [Dir::Up]);
    assert_eq!(step(&stix, &board(), Dir::Up), Some(Point::new(3, H - 1)));
}

#[test]
fn a_stix_cannot_start_into_a_claimed_area() {
    let mut board = board();
    board.set_cell(2, H - 2, Cell::Claimed(FAST));
    board.set_cell(2, H - 1, Cell::Claimed(FAST));
    let stix = Stix::new(Point::new(3, H - 1));

    assert_eq!(step(&stix, &board, Dir::Left), None);
    assert_eq!(legal_moves(&marker_on(&stix), &stix, &board), [Dir::Right]);
}

#[test]
fn crossing_the_stix_is_refused() {
    let stix = drawn(&[Dir::Up, Dir::Up, Dir::Up, Dir::Left, Dir::Down]);

    assert!(stix.contains_point(Point::new(3, H - 2)));
    assert_eq!(step(&stix, &board(), Dir::Right), None);
}

#[test]
fn retracing_the_last_segment_after_a_turn_is_refused() {
    let stix = drawn(&[Dir::Up, Dir::Up, Dir::Left]);

    assert_eq!(step(&stix, &board(), Dir::Right), None);
}

#[test]
fn a_u_turn_is_refused() {
    let stix = drawn(&[Dir::Up, Dir::Up]);

    assert_eq!(step(&stix, &board(), Dir::Down), None);
    assert_eq!(
        legal_moves(&marker_on(&stix), &stix, &board()),
        [Dir::Up, Dir::Left, Dir::Right]
    );
}

#[test]
fn returning_to_the_start_over_the_stix_is_refused() {
    let stix = drawn(&[Dir::Up]);

    assert_eq!(step(&stix, &board(), Dir::Down), None);
}

#[test]
fn closing_on_the_border_is_accepted() {
    let stix = drawn(&[Dir::Up, Dir::Up, Dir::Up, Dir::Up]);
    let top = Point::new(3, 0);

    assert_eq!(step(&stix, &board(), Dir::Up), Some(top));
    assert!(board().is_border(top));
}

#[test]
fn closing_on_a_claimed_area_is_accepted() {
    let mut board = board();
    board.set_cell(0, 0, Cell::Claimed(FAST));
    let stix = drawn_with(Point::new(1, H), &[Dir::Up; 3], FAST);

    assert_eq!(step(&stix, &board, Dir::Up), Some(Point::new(1, 1)));
}

#[test]
fn closing_on_the_start_over_a_new_edge_is_accepted() {
    let mut board = board();
    board.set_cell(0, 0, Cell::Claimed(FAST));
    let corner = Point::new(1, 1);
    let around = [Dir::Right, Dir::Down, Dir::Left];
    let stix = drawn_with(corner, &around, FAST);

    assert_eq!(step(&stix, &board, Dir::Up), Some(corner));
}

#[test]
fn a_dead_end_has_no_legal_move() {
    let path = [
        Dir::Up,
        Dir::Up,
        Dir::Up,
        Dir::Left,
        Dir::Left,
        Dir::Down,
        Dir::Down,
        Dir::Right,
        Dir::Up,
    ];
    let stix = drawn(&path);

    assert_eq!(stix.head(), Point::new(2, H - 2));
    assert!(legal_moves(&marker_on(&stix), &stix, &board()).is_empty());
}

#[test]
fn towards_head_follows_the_segments_round_their_corners() {
    let stix = drawn(&[Dir::Up, Dir::Up, Dir::Right, Dir::Up]);
    let corner = Point::new(START.x, H - 2);

    assert_eq!(stix.towards_head(START), Some(Dir::Up));
    assert_eq!(stix.towards_head(Point::new(START.x, H - 1)), Some(Dir::Up));
    assert_eq!(stix.towards_head(corner), Some(Dir::Right));
    assert_eq!(
        stix.towards_head(Point::new(START.x + 1, H - 2)),
        Some(Dir::Up)
    );
}

#[test]
fn towards_head_is_none_at_the_head_and_off_the_stix() {
    let stix = drawn(&[Dir::Up, Dir::Up]);

    assert_eq!(stix.towards_head(stix.head()), None);
    assert_eq!(stix.towards_head(Point::new(START.x + 1, H - 1)), None);
}
