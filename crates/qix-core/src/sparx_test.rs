use std::collections::HashSet;

use crate::qix_test::qix_at;
use crate::sparx::{nearest_border_point, turn};
use crate::{Board, Cell, Dir, DrawKind, Point, Rng, Sparx, SparxKind, Stix};

/// Dimensions of the small board most tests use.
const W: u16 = 6;
const H: u16 = 4;

/// Seed of the generator the tests draw from.
const SEED: u64 = 1195;

/// Steps the random-turn test draws, enough to see every way out.
const DRAWS: usize = 100;

const CLAIMED: Cell = Cell::Claimed(DrawKind::Fast);

fn sparx(pos: Point, dir: Dir) -> Sparx {
    Sparx {
        pos,
        dir,
        kind: SparxKind::Normal,
    }
}

/// A `W x H` board with the cells `claimed` lists claimed.
fn board_with(claimed: &[(u16, u16)]) -> Board {
    let mut board = Board::new(W, H);
    for &(x, y) in claimed {
        board.set_cell(x, y, CLAIMED);
    }
    board
}

/// The points a Sparx steps through on `board`, `steps` of them, starting
/// with the one after its own.
fn walk(board: &Board, mut sparx: Sparx, steps: usize) -> Vec<Point> {
    let mut rng = Rng::new(SEED);
    (0..steps)
        .map(|_| {
            sparx.step(board, &mut rng);
            sparx.pos
        })
        .collect()
}

#[test]
fn a_pair_spawns_at_the_top_centre_heading_opposite_ways() {
    let [left, right] = Sparx::spawn_pair(&Board::new(W, H));

    assert_eq!(left.pos, Point::new(W / 2, 0));
    assert_eq!(right.pos, left.pos);
    assert_eq!((left.dir, right.dir), (Dir::Left, Dir::Right));
    assert_eq!(
        (left.kind, right.kind),
        (SparxKind::Normal, SparxKind::Normal)
    );
}

#[test]
fn a_pair_spawns_on_the_border_nearest_the_top_centre() {
    let board = board_with(&[(2, 0), (3, 0), (2, 1), (3, 1)]);

    let [left, right] = Sparx::spawn_pair(&board);

    // (2, 0) and (4, 0) are one step away; (2, 0) comes first.
    assert!(!board.is_border(Point::new(W / 2, 0)));
    assert_eq!(left.pos, Point::new(W / 2 - 1, 0));
    assert_eq!(right.pos, left.pos);
}

#[test]
fn walks_the_fresh_border_in_a_loop_both_ways() {
    let board = Board::new(W, H);
    let perimeter = 2 * usize::from(W + H);

    for start in Sparx::spawn_pair(&board) {
        let points = walk(&board, start, perimeter);

        assert!(points.iter().all(|&point| board.is_border(point)));
        assert_eq!(points.iter().collect::<HashSet<_>>().len(), perimeter);
        assert_eq!(points.last(), Some(&start.pos));
    }
}

#[test]
fn each_way_round_visits_the_other_way_s_points_in_reverse() {
    let board = Board::new(W, H);
    let perimeter = 2 * usize::from(W + H);
    let [left, right] = Sparx::spawn_pair(&board);

    let mut leftwards = walk(&board, left, perimeter - 1);
    leftwards.reverse();

    assert_eq!(leftwards, walk(&board, right, perimeter - 1));
}

#[test]
fn turns_a_corner_rather_than_reversing() {
    let mut sparx = sparx(Point::new(0, 0), Dir::Left);

    sparx.step(&Board::new(W, H), &mut Rng::new(SEED));

    assert_eq!((sparx.pos, sparx.dir), (Point::new(0, 1), Dir::Down));
}

#[test]
fn goes_straight_through_a_junction() {
    // Diagonal claimed cells meet at (2, 2), where four border edges meet.
    let board = board_with(&[(1, 1), (2, 2)]);
    let junction = Point::new(2, 2);
    assert_eq!(board.border_neighbours(junction).count(), Dir::ALL.len());

    for dir in Dir::ALL {
        let mut sparx = sparx(junction, dir);
        sparx.step(&board, &mut Rng::new(SEED));
        assert_eq!((Some(sparx.pos), sparx.dir), (dir.step(junction), dir));
    }
}

#[test]
fn turns_a_random_way_when_it_cannot_go_straight() {
    // On the top edge, heading down: left and right are the ways out.
    let board = Board::new(W, H);
    let start = Point::new(W / 2, 0);
    let mut rng = Rng::new(SEED);

    let dirs: HashSet<Dir> = (0..DRAWS)
        .map(|_| {
            let mut sparx = sparx(start, Dir::Down);
            sparx.step(&board, &mut rng);
            sparx.dir
        })
        .collect();

    assert_eq!(dirs, HashSet::from([Dir::Left, Dir::Right]));
}

#[test]
fn turn_prefers_straight_then_sideways_and_reverses_only_at_a_dead_end() {
    let mut rng = Rng::new(SEED);
    let edges = |dirs: &'static [Dir]| move || dirs.iter().copied();

    let straight = turn(
        Dir::Right,
        edges(&[Dir::Left, Dir::Up, Dir::Right]),
        &mut rng,
    );
    let sideways = turn(Dir::Right, edges(&[Dir::Left, Dir::Up]), &mut rng);
    let dead_end = turn(Dir::Right, edges(&[Dir::Left]), &mut rng);
    let isolated = turn(Dir::Right, edges(&[]), &mut rng);

    assert_eq!(straight, Some(Dir::Right));
    assert_eq!(sideways, Some(Dir::Up));
    assert_eq!(dead_end, Some(Dir::Left));
    assert_eq!(isolated, None);
}

#[test]
fn a_choice_free_turn_draws_nothing() {
    let mut rng = Rng::new(SEED);
    let edges = || [Dir::Left, Dir::Up].into_iter();

    turn(Dir::Right, edges, &mut rng);

    assert_eq!(rng, Rng::new(SEED));
}

#[test]
fn snaps_to_the_nearest_border_point_after_a_claim() {
    // A stix straight down column 2 claims the cells left of it.
    let mut board = Board::new(W, H);
    let mut stix = Stix::new(Point::new(2, 0));
    stix.push(Point::new(2, H), DrawKind::Fast);
    board.close_stix(&stix, &[qix_at((W - 1, H / 2))]);
    let mut cut_off = sparx(Point::new(0, 1), Dir::Down);

    cut_off.snap_to_border(&board);

    assert_eq!(cut_off.pos, Point::new(2, 1));
    assert_eq!(cut_off.dir, Dir::Down);
}

#[test]
fn a_sparx_still_on_the_border_does_not_snap() {
    let board = board_with(&[(0, 0)]);
    let on_border = sparx(Point::new(W, 1), Dir::Up);
    let mut snapped = on_border;

    snapped.snap_to_border(&board);

    assert_eq!(snapped, on_border);
}

#[test]
fn the_nearest_border_point_is_the_first_of_equally_near_ones() {
    // (1, 0) is two steps from both (3, 0) and (1, 2).
    let board = board_with(&[(0, 0), (1, 0), (2, 0), (0, 1), (1, 1), (2, 1)]);

    let nearest = nearest_border_point(&board, Point::new(1, 0));

    assert_eq!(nearest, Some(Point::new(3, 0)));
}

#[test]
fn a_board_with_no_border_has_no_nearest_border_point() {
    let mut board = Board::new(1, 1);
    board.set_cell(0, 0, CLAIMED);

    assert_eq!(nearest_border_point(&board, Point::new(0, 0)), None);
}

#[test]
fn a_sparx_stepping_onto_the_marker_hits_it() {
    let marker = Point::new(2, 0);
    let sparx = sparx(marker, Dir::Left);

    assert!(sparx.hits_marker(Point::new(3, 0), marker, marker));
}

#[test]
fn the_marker_stepping_onto_a_sparx_hits_it() {
    let at = Point::new(2, 0);
    let sparx = sparx(at, Dir::Left);

    assert!(sparx.hits_marker(at, Point::new(1, 0), at));
}

#[test]
fn a_sparx_and_the_marker_swapping_points_hit() {
    let (a, b) = (Point::new(1, 0), Point::new(2, 0));
    let sparx = sparx(a, Dir::Left);

    assert!(sparx.hits_marker(b, a, b));
}

#[test]
fn a_sparx_following_the_marker_does_not_hit_it() {
    // The marker goes from 2 to 3 while the Sparx goes from 1 to 2.
    let sparx = sparx(Point::new(2, 0), Dir::Right);

    assert!(!sparx.hits_marker(Point::new(1, 0), Point::new(2, 0), Point::new(3, 0)));
}

#[test]
fn a_sparx_next_to_the_marker_does_not_hit_it() {
    let sparx = sparx(Point::new(3, 0), Dir::Right);
    let marker = Point::new(2, 0);

    assert!(!sparx.hits_marker(Point::new(3, 0), marker, marker));
}

#[test]
fn spawn_at_top_alternates_left_and_right_from_the_pair_point() {
    let board = board_with(&[(W / 2 - 1, 0), (W / 2, 0)]);
    let [pair, _] = Sparx::spawn_pair(&board);

    let spawned: Vec<_> = Sparx::spawn_at_top(&board, 3).collect();

    assert!(spawned.iter().all(|sparx| sparx.pos == pair.pos));
    assert!(spawned.iter().all(|sparx| sparx.kind == SparxKind::Normal));
    let dirs: Vec<_> = spawned.iter().map(|sparx| sparx.dir).collect();
    assert_eq!(dirs, [Dir::Left, Dir::Right, Dir::Left]);
}
