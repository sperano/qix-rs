use crate::tuning::{BOARD_H, BOARD_W, MARKER_STEP_TICKS};
use crate::{Board, Cell, Dir, DrawKind, Game, InputState, MarkerState, Point};

/// Dimensions of the small board the lap and corner tests use.
const W: u16 = 4;
const H: u16 = 3;

/// Steps the marker-timing tests hold a direction for.
const STEPS: u32 = 5;

/// Lattice steps that take the marker past any edge of the full board.
const BEYOND_ANY_EDGE: u32 = BOARD_W as u32 + 1;

fn holding(dir: Dir) -> InputState {
    InputState {
        dir: Some(dir),
        ..InputState::default()
    }
}

/// Ticks `game` `ticks` times with the same input.
fn hold(game: &mut Game, input: InputState, ticks: u32) {
    for _ in 0..ticks {
        game.tick(&input);
    }
}

/// Ticks `game` long enough for `steps` marker steps towards `dir`.
fn walk(game: &mut Game, dir: Dir, steps: u32) {
    hold(game, holding(dir), steps * MARKER_STEP_TICKS);
}

#[test]
fn new_game_has_not_ticked() {
    assert_eq!(Game::new().ticks(), 0);
}

#[test]
fn each_tick_advances_one_step() {
    let mut game = Game::new();

    game.tick(&holding(Dir::Left));
    game.tick(&InputState::default());

    assert_eq!(game.ticks(), 2);
}

#[test]
fn new_game_has_a_free_board_and_the_marker_at_the_bottom_centre() {
    let game = Game::new();

    assert_eq!(game.board(), &Board::new(BOARD_W, BOARD_H));
    assert_eq!(game.marker().pos, Point::new(BOARD_W / 2, BOARD_H));
    assert_eq!(game.marker().state, MarkerState::OnBorder);
}

#[test]
fn marker_moves_exactly_one_step_per_marker_step_ticks() {
    let mut game = Game::new();
    let start = game.marker().pos;

    for tick in 1..=STEPS * MARKER_STEP_TICKS {
        game.tick(&holding(Dir::Right));
        let steps = u16::try_from(tick / MARKER_STEP_TICKS).unwrap();
        assert_eq!(game.marker().pos, Point::new(start.x + steps, start.y));
    }
}

#[test]
fn marker_stays_put_without_a_direction() {
    let mut game = Game::new();
    let start = game.marker().pos;

    hold(&mut game, InputState::default(), STEPS * MARKER_STEP_TICKS);

    assert_eq!(game.marker().pos, start);
}

#[test]
fn releasing_the_direction_restarts_the_step_count() {
    let mut game = Game::new();
    let start = game.marker().pos;

    hold(&mut game, holding(Dir::Right), MARKER_STEP_TICKS - 1);
    game.tick(&InputState::default());
    hold(&mut game, holding(Dir::Right), MARKER_STEP_TICKS - 1);

    assert_eq!(game.marker().pos, start);
}

#[test]
fn marker_cannot_leave_the_border_without_a_draw_button() {
    let mut game = Game::new();
    let ticks = BEYOND_ANY_EDGE * MARKER_STEP_TICKS;

    for dir in [Dir::Up, Dir::Left, Dir::Up, Dir::Right, Dir::Down] {
        for _ in 0..ticks {
            game.tick(&holding(dir));
            assert!(game.board().is_border(game.marker().pos), "{dir:?}");
        }
    }
}

#[test]
fn marker_stops_at_the_edge_it_walks_along() {
    let mut game = Game::new();

    walk(&mut game, Dir::Right, BEYOND_ANY_EDGE);

    assert_eq!(game.marker().pos, Point::new(BOARD_W, BOARD_H));
}

#[test]
fn marker_follows_the_border_with_a_draw_button_held() {
    let mut game = Game::new();
    let start = game.marker().pos;
    let input = InputState {
        fast: true,
        ..holding(Dir::Left)
    };

    hold(&mut game, input, STEPS * MARKER_STEP_TICKS);

    let steps = u16::try_from(STEPS).unwrap();
    assert_eq!(game.marker().pos, Point::new(start.x - steps, start.y));
}

#[test]
fn marker_turns_every_outer_corner_on_a_lap() {
    let mut game = Game::with_board(Board::new(W, H));
    let start = game.marker().pos;
    let (w, h) = (u32::from(W), u32::from(H));
    let half = u32::from(W / 2);
    let lap = [
        (Dir::Right, w - half, Point::new(W, H)),
        (Dir::Up, h, Point::new(W, 0)),
        (Dir::Left, w, Point::new(0, 0)),
        (Dir::Down, h, Point::new(0, H)),
        (Dir::Right, half, start),
    ];

    for (dir, steps, corner) in lap {
        walk(&mut game, dir, steps);
        assert_eq!(game.marker().pos, corner, "{dir:?}");
    }
}

#[test]
fn marker_turns_the_corners_around_a_claimed_cell() {
    let mut board = Board::new(W, H);
    board.set_cell(W - 1, H - 1, Cell::Claimed(DrawKind::Fast));
    let mut game = Game::with_board(board);
    let path = [
        (Dir::Right, Point::new(W - 1, H)),
        (Dir::Right, Point::new(W - 1, H)),
        (Dir::Up, Point::new(W - 1, H - 1)),
        (Dir::Up, Point::new(W - 1, H - 1)),
        (Dir::Right, Point::new(W, H - 1)),
        (Dir::Up, Point::new(W, H - 2)),
    ];

    for (dir, pos) in path {
        walk(&mut game, dir, 1);
        assert_eq!(game.marker().pos, pos, "{dir:?}");
    }
}
