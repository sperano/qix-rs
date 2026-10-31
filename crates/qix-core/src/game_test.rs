use crate::qix_test::qix_at;
use crate::tuning::{BOARD_H, BOARD_W, MARKER_STEP_TICKS, SLOW_STEP_TICKS};
use crate::{Board, Cell, Dir, DrawKind, Game, InputState, MarkerState, Point, Qix, Rng, Vec2};

mod claims;
mod deaths;
mod levels;
mod qix_hits;
mod sparx_hits;

/// Seed of every game the tests play.
const SEED: u64 = 1194;

/// Dimensions of the small board the lap and corner tests use.
const W: u16 = 4;
const H: u16 = 3;

/// Dimensions of the board of [`reaching_the_border_closes_the_stix`],
/// tall enough to pen its Qix in [`PEN_COLUMN`].
const PEN_W: u16 = 8;
const PEN_H: u16 = 6;

/// The column [`penned`] keeps the Qix of
/// [`reaching_the_border_closes_the_stix`] in, clear of the stix.
const PEN_COLUMN: u16 = 1;

/// Steps the marker-timing tests hold a direction for.
const STEPS: u32 = 5;

/// Lattice steps that take the marker past any edge of the full board.
const BEYOND_ANY_EDGE: u32 = BOARD_W as u32 + 1;

/// A new full-size game.
fn new_game() -> Game {
    Game::new(Rng::new(SEED))
}

/// A game on `board` with a short Qix in its top-left cell, which must be
/// free.
fn small_game(board: Board) -> Game {
    Game::with_board(board, qix_at((0, 0)), Vec::new(), Rng::new(SEED))
}

/// A free `width x height` board but for claimed walls beside `column`
/// from the second row to the second last, and a vertical Qix down the
/// middle of the column, too long to turn between its walls: it stays in
/// the column however it moves, yet a claim floods from it into the rest
/// of the board through the first and last rows.
pub(crate) fn penned(width: u16, height: u16, column: u16) -> (Board, Qix) {
    let mut board = Board::new(width, height);
    for y in 1..height - 1 {
        for x in [column.checked_sub(1), Some(column + 1)]
            .into_iter()
            .flatten()
        {
            if x < width {
                board.set_cell(x, y, Cell::Claimed(DrawKind::Fast));
            }
        }
    }
    let x = f32::from(column) + 0.5;
    let (top, bottom) = (0.5, f32::from(height) - 1.5);
    let qix = Qix::new(
        Vec2::new(x, top),
        Vec2::new(x, bottom),
        Vec2::default(),
        0.0,
    );
    (board, qix)
}

fn holding(dir: Dir) -> InputState {
    InputState {
        dir: Some(dir),
        ..InputState::default()
    }
}

/// `dir` held with the draw button of `kind`.
fn drawing(dir: Dir, kind: DrawKind) -> InputState {
    InputState {
        fast: kind == DrawKind::Fast,
        slow: kind == DrawKind::Slow,
        ..holding(dir)
    }
}

/// The ticks one step of a stix drawn with `kind` takes.
fn draw_step_ticks(kind: DrawKind) -> u32 {
    match kind {
        DrawKind::Fast => MARKER_STEP_TICKS,
        DrawKind::Slow => SLOW_STEP_TICKS,
    }
}

/// Ticks `game` long enough for `steps` steps of a stix drawn towards
/// `dir` with `kind`.
fn draw(game: &mut Game, dir: Dir, kind: DrawKind, steps: u32) {
    hold(game, drawing(dir, kind), steps * draw_step_ticks(kind));
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
    assert_eq!(new_game().ticks(), 0);
}

#[test]
fn each_tick_advances_one_step() {
    let mut game = new_game();

    game.tick(&holding(Dir::Left));
    game.tick(&InputState::default());

    assert_eq!(game.ticks(), 2);
}

#[test]
fn new_game_has_a_free_board_and_the_marker_at_the_bottom_centre() {
    let game = new_game();

    assert_eq!(game.board(), &Board::new(BOARD_W, BOARD_H));
    assert_eq!(game.marker().pos, Point::new(BOARD_W / 2, BOARD_H));
    assert_eq!(game.marker().state, MarkerState::OnBorder);
}

#[test]
fn marker_moves_exactly_one_step_per_marker_step_ticks() {
    let mut game = new_game();
    let start = game.marker().pos;

    for tick in 1..=STEPS * MARKER_STEP_TICKS {
        game.tick(&holding(Dir::Right));
        let steps = u16::try_from(tick / MARKER_STEP_TICKS).unwrap();
        assert_eq!(game.marker().pos, Point::new(start.x + steps, start.y));
    }
}

#[test]
fn marker_stays_put_without_a_direction() {
    let mut game = new_game();
    let start = game.marker().pos;

    hold(&mut game, InputState::default(), STEPS * MARKER_STEP_TICKS);

    assert_eq!(game.marker().pos, start);
}

#[test]
fn releasing_the_direction_restarts_the_step_count() {
    let mut game = new_game();
    let start = game.marker().pos;

    hold(&mut game, holding(Dir::Right), MARKER_STEP_TICKS - 1);
    game.tick(&InputState::default());
    hold(&mut game, holding(Dir::Right), MARKER_STEP_TICKS - 1);

    assert_eq!(game.marker().pos, start);
}

#[test]
fn marker_cannot_leave_the_border_without_a_draw_button() {
    let mut game = new_game();
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
    let mut game = new_game();

    walk(&mut game, Dir::Right, BEYOND_ANY_EDGE);

    assert_eq!(game.marker().pos, Point::new(BOARD_W, BOARD_H));
}

#[test]
fn marker_follows_the_border_with_a_draw_button_held() {
    let mut game = new_game();
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
    let mut game = small_game(Board::new(W, H));
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
    let mut game = small_game(board);
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

#[test]
fn a_draw_button_and_a_move_into_free_space_start_a_stix() {
    let mut game = small_game(Board::new(W, H));
    let start = game.marker().pos;

    draw(&mut game, Dir::Up, DrawKind::Fast, 1);

    let head = Point::new(start.x, start.y - 1);
    assert_eq!(game.marker().pos, head);
    assert_eq!(game.marker().state, MarkerState::Drawing);
    let stix = game.stix().unwrap();
    assert_eq!(stix.points(), [start, head]);
    assert_eq!(stix.kinds(), [DrawKind::Fast]);
}

#[test]
fn each_draw_button_steps_at_its_speed() {
    for kind in [DrawKind::Fast, DrawKind::Slow] {
        let mut game = new_game();
        let start = game.marker().pos;
        let step_ticks = draw_step_ticks(kind);

        for tick in 1..=STEPS * step_ticks {
            game.tick(&drawing(Dir::Up, kind));
            let steps = u16::try_from(tick / step_ticks).unwrap();
            assert_eq!(
                game.marker().pos,
                Point::new(start.x, start.y - steps),
                "{kind:?}"
            );
        }
    }
}

#[test]
fn the_slow_button_follows_the_border_at_marker_speed() {
    let mut game = new_game();
    let start = game.marker().pos;

    hold(
        &mut game,
        drawing(Dir::Left, DrawKind::Slow),
        STEPS * MARKER_STEP_TICKS,
    );

    let steps = u16::try_from(STEPS).unwrap();
    assert_eq!(game.marker().pos, Point::new(start.x - steps, start.y));
    assert_eq!(game.marker().state, MarkerState::OnBorder);
}

#[test]
fn releasing_both_draw_buttons_stops_the_marker_on_the_stix() {
    let mut game = new_game();
    draw(&mut game, Dir::Up, DrawKind::Fast, 1);
    let head = game.marker().pos;

    walk(&mut game, Dir::Up, STEPS);

    assert_eq!(game.marker().pos, head);
    assert_eq!(game.marker().state, MarkerState::Drawing);
    assert_eq!(game.stix().unwrap().head(), head);
}

#[test]
fn pressing_a_draw_button_again_resumes_drawing() {
    let mut game = new_game();
    draw(&mut game, Dir::Up, DrawKind::Fast, 1);
    let head = game.marker().pos;
    hold(
        &mut game,
        drawing(Dir::Up, DrawKind::Fast),
        MARKER_STEP_TICKS - 1,
    );
    game.tick(&holding(Dir::Up));

    hold(
        &mut game,
        drawing(Dir::Up, DrawKind::Slow),
        SLOW_STEP_TICKS - 1,
    );
    assert_eq!(game.marker().pos, head);
    game.tick(&drawing(Dir::Up, DrawKind::Slow));

    assert_eq!(game.marker().pos, Point::new(head.x, head.y - 1));
    assert_eq!(
        game.stix().unwrap().kinds(),
        [DrawKind::Fast, DrawKind::Slow]
    );
}

#[test]
fn a_single_fast_segment_makes_the_claim_fast() {
    let mut game = new_game();

    draw(&mut game, Dir::Up, DrawKind::Slow, 2);
    draw(&mut game, Dir::Left, DrawKind::Fast, 1);
    draw(&mut game, Dir::Up, DrawKind::Slow, 1);

    let stix = game.stix().unwrap();
    assert_eq!(
        stix.kinds(),
        [DrawKind::Slow, DrawKind::Fast, DrawKind::Slow]
    );
    assert_eq!(stix.claim_kind(), DrawKind::Fast);
}

#[test]
fn a_stix_drawn_slow_throughout_claims_slow() {
    let mut game = new_game();

    draw(&mut game, Dir::Up, DrawKind::Slow, 2);
    draw(&mut game, Dir::Right, DrawKind::Slow, 1);

    assert_eq!(game.stix().unwrap().claim_kind(), DrawKind::Slow);
}

#[test]
fn an_illegal_draw_step_leaves_the_marker_drawing_where_it_is() {
    let mut game = new_game();
    draw(&mut game, Dir::Up, DrawKind::Fast, 2);
    let stix = game.stix().unwrap().clone();

    draw(&mut game, Dir::Down, DrawKind::Fast, 1);

    assert_eq!(game.marker().pos, stix.head());
    assert_eq!(game.marker().state, MarkerState::Drawing);
    assert_eq!(game.stix(), Some(&stix));
}

#[test]
fn a_draw_step_off_the_board_starts_no_stix() {
    let mut game = new_game();
    let start = game.marker().pos;

    draw(&mut game, Dir::Down, DrawKind::Fast, 1);

    assert_eq!(game.marker().pos, start);
    assert_eq!(game.marker().state, MarkerState::OnBorder);
    assert_eq!(game.stix(), None);
}

#[test]
fn reaching_the_border_closes_the_stix() {
    let (board, qix) = penned(PEN_W, PEN_H, PEN_COLUMN);
    let mut game = Game::with_board(board, qix, Vec::new(), Rng::new(SEED));
    let start = game.marker().pos;

    draw(&mut game, Dir::Up, DrawKind::Fast, u32::from(PEN_H));

    assert_eq!(game.marker().pos, Point::new(start.x, 0));
    assert_eq!(game.marker().state, MarkerState::OnBorder);
    assert_eq!(game.stix(), None);
    walk(&mut game, Dir::Down, 1);
    assert_eq!(game.marker().pos, Point::new(start.x, 1));
}

/// The claimed cells of `board` in reading order.
fn claimed_cells(board: &Board) -> Vec<(u16, u16)> {
    let (width, height) = (board.width(), board.height());
    (0..height)
        .flat_map(|y| (0..width).map(move |x| (x, y)))
        .filter(|&(x, y)| board.cell(i32::from(x), i32::from(y)).is_claimed())
        .collect()
}

/// Draws the L-shaped stix on the `W x H` board: up one step from the
/// marker's start, then left to the left edge, which closes it and claims
/// the cells below it. Returns the game and the stix's lattice points.
fn closed_l() -> (Game, Vec<Point>) {
    let mut game = small_game(Board::new(W, H));
    let start = game.marker().pos;
    draw(&mut game, Dir::Up, DrawKind::Fast, 1);
    draw(&mut game, Dir::Left, DrawKind::Fast, u32::from(start.x));
    let corner_y = start.y - 1;
    let mut points = vec![start];
    points.extend((0..=start.x).rev().map(|x| Point::new(x, corner_y)));
    (game, points)
}

#[test]
fn the_former_stix_points_are_all_border_points() {
    let (game, points) = closed_l();

    assert_eq!(game.stix(), None);
    let (start, corner) = (points[0], points[1]);
    let below_l: Vec<_> = (corner.y..H)
        .flat_map(|y| (0..start.x).map(move |x| (x, y)))
        .collect();
    assert_eq!(claimed_cells(game.board()), below_l);
    for point in points {
        assert!(game.board().is_border(point), "{point:?}");
    }
}

#[test]
fn the_marker_ends_on_the_border_at_the_closing_point() {
    let (game, points) = closed_l();

    assert_eq!(game.marker().pos, *points.last().unwrap());
    assert_eq!(game.marker().state, MarkerState::OnBorder);
}

#[test]
fn the_marker_walks_the_new_border_back_to_the_stix_start() {
    let (mut game, points) = closed_l();
    let (start, corner) = (points[0], points[1]);

    walk(&mut game, Dir::Right, u32::from(corner.x));
    assert_eq!(game.marker().pos, corner);
    walk(&mut game, Dir::Down, 1);

    assert_eq!(game.marker().pos, start);
    assert_eq!(game.marker().state, MarkerState::OnBorder);
}
