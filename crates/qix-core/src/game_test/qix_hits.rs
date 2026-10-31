//! The Qix killing the player through [`Game::tick`].

use super::{SEED, drawing};
use crate::collision::qix_hits_marker;
use crate::tuning::MARKER_STEP_TICKS;
use crate::{Board, Cell, DeathCause, Dir, DrawKind, Event, Game, InputState, Qix, Rng, Vec2};

/// Dimensions of the boards: the marker starts at `(W / 2, H)`.
const W: u16 = 20;
const H: u16 = 30;

/// Rows of free cells along the bottom edge of [`band_and_column`].
const BAND_ROWS: u16 = 3;

/// The free column of [`band_and_column`], right of the marker's start.
const COLUMN: u16 = 15;

/// Steps the stix goes up into the band before turning right.
const STEPS_UP: u32 = 2;

/// Steps the marker goes past [`COLUMN`], far enough that the Qix in it
/// stays clear of the marker's square.
const CLEAR_OF_COLUMN: u32 = 3;

/// Ticks the tests watch a standing marker or Qix for.
const WATCH_TICKS: u32 = 60;

/// A `W x H` board with only the cells `free` keeps free.
fn board_with(free: impl Fn(u16, u16) -> bool) -> Board {
    let mut board = Board::new(W, H);
    for y in 0..H {
        for x in 0..W {
            if !free(x, y) {
                board.set_cell(x, y, Cell::Claimed(DrawKind::Fast));
            }
        }
    }
    board
}

/// A vertical Qix `length` long, down from `(x, top)`: on a free column
/// almost as long, the claimed cells around it keep it there while it
/// ticks.
fn column_qix(x: f32, top: f32, length: f32) -> Qix {
    let (a, b) = (Vec2::new(x, top), Vec2::new(x, top + length));
    Qix::new(a, b, Vec2::default(), 0.0)
}

/// A band of free rows along the bottom edge and a free column from it up
/// to the top edge, with a Qix along the column that crosses every row of
/// the band.
fn band_and_column() -> Game {
    let board = board_with(|x, y| y >= H - BAND_ROWS || x == COLUMN);
    let qix = column_qix(f32::from(COLUMN) + 0.5, 0.5, f32::from(H) - 1.0);
    Game::with_board(board, qix, Vec::new(), Rng::new(SEED))
}

/// Ticks `game` `ticks` times with `input` and returns the deaths raised.
fn deaths(game: &mut Game, input: InputState, ticks: u32) -> Vec<Event> {
    let mut deaths = Vec::new();
    for _ in 0..ticks {
        game.tick(&input);
        deaths.extend_from_slice(game.events());
    }
    deaths
}

/// Draws a stix up into the band, then `right` steps to the right, and
/// returns the deaths raised on the way.
fn draw_up_then_right(game: &mut Game, right: u32) -> Vec<Event> {
    let kind = DrawKind::Fast;
    let mut raised = deaths(game, drawing(Dir::Up, kind), STEPS_UP * MARKER_STEP_TICKS);
    raised.extend(deaths(
        game,
        drawing(Dir::Right, kind),
        right * MARKER_STEP_TICKS,
    ));
    raised
}

#[test]
fn a_qix_crossing_a_horizontal_stix_kills() {
    let mut game = band_and_column();
    let start_x = u32::from(game.marker().pos.x);
    let past_column = u32::from(COLUMN) + CLEAR_OF_COLUMN - start_x;
    draw_up_then_right(&mut game, past_column);

    game.tick(&InputState::default());

    assert!(game.stix().is_some(), "the stix closed");
    assert_eq!(
        u32::from(game.marker().pos.x),
        u32::from(COLUMN) + CLEAR_OF_COLUMN
    );
    assert!(
        !qix_hits_marker(game.qix(), game.marker().pos),
        "the Qix touches the marker"
    );
    assert_eq!(game.events(), [Event::Death(DeathCause::Qix)]);
}

#[test]
fn a_qix_beside_the_stix_does_not_kill() {
    let mut game = band_and_column();
    let start_x = u32::from(game.marker().pos.x);
    let short_of_column = u32::from(COLUMN) - 1 - start_x;

    let mut raised = draw_up_then_right(&mut game, short_of_column);
    raised.extend(deaths(&mut game, InputState::default(), WATCH_TICKS));

    assert!(game.stix().is_some(), "the stix closed");
    assert_eq!(game.marker().pos.x, COLUMN - 1);
    assert_eq!(raised, []);
}

#[test]
fn a_qix_next_to_a_marker_on_the_border_does_not_kill() {
    let column = W / 2;
    let board = board_with(|x, _| x == column);
    let length = f32::from(H) - 0.5;
    let qix = column_qix(f32::from(column) + 0.25, 0.25, length);
    let mut game = Game::with_board(board, qix, Vec::new(), Rng::new(SEED));
    let mut touching_ticks = 0;

    for _ in 0..WATCH_TICKS {
        game.tick(&InputState::default());
        assert_eq!(game.events(), []);
        touching_ticks += u32::from(qix_hits_marker(game.qix(), game.marker().pos));
    }

    assert!(touching_ticks > 0, "the Qix never came near the marker");
}
