//! The Qix killing the player through [`Game::tick`].

use super::{SEED, drawing};
use crate::collision::qix_hits_marker;
use crate::tuning::MARKER_STEP_TICKS;
use crate::{
    Board, Cell, DeathCause, Dir, DrawKind, Event, Game, InputState, Point, Qix, Rng, Sparx, Stix,
    Vec2,
};

/// Dimensions of the boards: the marker starts at `(W / 2, H)`.
const W: u16 = 20;
const H: u16 = 30;

/// Rows of free cells along the bottom edge of [`band_and_column`].
const BAND_ROWS: u16 = 3;

/// The free column of [`band_and_column`], right of the marker's start.
pub(super) const COLUMN: u16 = 15;

/// Steps the stix goes up into the band before turning right.
pub(super) const STEPS_UP: u32 = 2;

/// Steps the marker goes past [`COLUMN`], far enough that the Qix in it
/// stays clear of the marker's square.
const CLEAR_OF_COLUMN: u32 = 3;

/// Where the top of [`falling_qix`]'s Qix starts, its length and the
/// speed it starts falling at, in playfield units (per tick).
const FALL_TOP: f32 = 10.0;
const FALL_LENGTH: f32 = 2.0;
const FALL_SPEED: f32 = 0.5;

/// A free column at the top of the board, closed off from the rest of
/// it: [`with_a_penned_qix`]'s other Qix is kept there, clear of every
/// stix.
const POCKET_COLUMN: u16 = 2;
const POCKET_ROWS: u16 = 10;

/// Ticks the tests watch a standing marker or Qix for.
const WATCH_TICKS: u32 = 60;

/// The free cells of [`closing_through_a_qix`]: a box under a channel one
/// row high and two cells wide, which the stix up the box's middle
/// reaches the border in.
const CLOSING_BOX: (std::ops::Range<u16>, std::ops::Range<u16>) = (W / 2 - 2..W / 2 + 2, H - 2..H);
const CLOSING_CHANNEL: (std::ops::Range<u16>, u16) = (W / 2 - 1..W / 2 + 1, H - 3);

/// Steps of the stix up from the marker's start to the channel.
const CLOSING_STEPS: u32 = 3;

/// How far the ends of [`closing_through_a_qix`]'s Qix are from the sides
/// of the channel, and how far down into it the Qix lies: too little room
/// to move at its slowest speed, and clear of the square of the marker
/// below the channel.
const CHANNEL_MARGIN: f32 = 0.1;
const CHANNEL_DEPTH: f32 = 0.25;

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
pub(super) fn band_and_column() -> Game {
    band_and_column_with(Vec::new())
}

/// [`band_and_column`] with `sparx`.
pub(super) fn band_and_column_with(sparx: Vec<Sparx>) -> Game {
    let board = board_with(|x, y| y >= H - BAND_ROWS || x == COLUMN);
    let qix = column_qix(f32::from(COLUMN) + 0.5, 0.5, f32::from(H) - 1.0);
    Game::with_board(board, vec![qix], sparx, Rng::new(SEED))
}

/// [`band_and_column`] with another Qix in a pocket of its own, either
/// before the column's Qix in the game's list or after it.
fn with_a_penned_qix(penned_first: bool) -> Game {
    let pocket = |x, y| x == POCKET_COLUMN && y < POCKET_ROWS;
    let board = board_with(|x, y| y >= H - BAND_ROWS || x == COLUMN || pocket(x, y));
    let length = f32::from(POCKET_ROWS) - 1.0;
    let penned = column_qix(f32::from(POCKET_COLUMN) + 0.5, 0.5, length);
    let column = column_qix(f32::from(COLUMN) + 0.5, 0.5, f32::from(H) - 1.0);
    let qix = if penned_first {
        vec![penned, column]
    } else {
        vec![column, penned]
    };
    Game::with_board(board, qix, Vec::new(), Rng::new(SEED))
}

/// A box of free cells around the bottom of the marker's column, under a
/// channel of two free cells either side of it, holding a level Qix that
/// fills the channel: it cannot move, and the stix up the column closes
/// with a step across it.
fn closing_through_a_qix() -> Game {
    let ((box_x, box_y), (channel_x, channel_y)) = (CLOSING_BOX, CLOSING_CHANNEL);
    let in_box = |x, y| box_x.contains(&x) && box_y.contains(&y);
    let board = board_with(|x, y| in_box(x, y) || (channel_x.contains(&x) && y == channel_y));
    let y = f32::from(channel_y) + CHANNEL_DEPTH;
    let a = Vec2::new(f32::from(channel_x.start) + CHANNEL_MARGIN, y);
    let b = Vec2::new(f32::from(channel_x.end) - CHANNEL_MARGIN, y);
    let qix = Qix::new(a, b, Vec2::default(), 0.0);
    Game::with_board(board, vec![qix], Vec::new(), Rng::new(SEED))
}

/// Ticks `game` `ticks` times with `input` and returns the deaths the
/// Qix raised: a marker standing still that long also lights the fuse.
fn deaths(game: &mut Game, input: InputState, ticks: u32) -> Vec<Event> {
    let mut deaths = Vec::new();
    for _ in 0..ticks {
        game.tick(&input);
        let by_qix = |&&event: &&Event| event == Event::Death(DeathCause::Qix);
        deaths.extend(game.events().iter().filter(by_qix));
    }
    deaths
}

/// [`band_and_column`] with a short Qix high up the column instead,
/// falling towards the band: it reaches the band only after a stix drawn
/// across the column has passed it.
fn falling_qix() -> Game {
    let board = board_with(|x, y| y >= H - BAND_ROWS || x == COLUMN);
    let qix = Qix::new(
        Vec2::new(f32::from(COLUMN) + 0.5, FALL_TOP),
        Vec2::new(f32::from(COLUMN) + 0.5, FALL_TOP + FALL_LENGTH),
        Vec2::new(0.0, FALL_SPEED),
        0.0,
    );
    Game::with_board(board, vec![qix], Vec::new(), Rng::new(SEED))
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
    let mut game = falling_qix();
    let start_x = u32::from(game.marker().pos.x);
    let past_column = u32::from(COLUMN) + CLEAR_OF_COLUMN - start_x;
    let mut raised = draw_up_then_right(&mut game, past_column);
    assert_eq!(
        raised,
        [],
        "the Qix reached the stix before the marker passed"
    );
    let head = game.marker().pos;

    let mut ticks = 0;
    while raised.is_empty() && ticks < WATCH_TICKS {
        raised = deaths(&mut game, InputState::default(), 1);
        ticks += 1;
    }

    assert_eq!(raised, [Event::Death(DeathCause::Qix)]);
    assert_eq!(u32::from(head.x), u32::from(COLUMN) + CLEAR_OF_COLUMN);
    assert!(
        !qix_hits_marker(&game.qix()[0], head),
        "the Qix touches the marker"
    );
}

#[test]
fn a_qix_beside_the_stix_does_not_kill() {
    let mut game = band_and_column();
    let start_x = u32::from(game.marker().pos.x);
    let short_of_column = u32::from(COLUMN) - 1 - start_x;

    let mut raised = draw_up_then_right(&mut game, short_of_column);
    assert!(game.stix().is_some(), "the stix closed");
    assert_eq!(game.marker().pos.x, COLUMN - 1);
    // The fuse may kill the standing marker meanwhile: `deaths` only
    // counts the Qix's.
    raised.extend(deaths(&mut game, InputState::default(), WATCH_TICKS));

    assert_eq!(raised, []);
}

#[test]
fn a_qix_next_to_a_marker_on_the_border_does_not_kill() {
    let column = W / 2;
    let board = board_with(|x, _| x == column);
    let length = f32::from(H) - 0.5;
    let qix = column_qix(f32::from(column) + 0.25, 0.25, length);
    let mut game = Game::with_board(board, vec![qix], Vec::new(), Rng::new(SEED));
    let mut touching_ticks = 0;

    for _ in 0..WATCH_TICKS {
        game.tick(&InputState::default());
        assert_eq!(game.events(), []);
        touching_ticks += u32::from(qix_hits_marker(&game.qix()[0], game.marker().pos));
    }

    assert!(touching_ticks > 0, "the Qix never came near the marker");
}

#[test]
fn either_of_two_qix_kills() {
    for penned_first in [true, false] {
        let mut game = with_a_penned_qix(penned_first);
        let start_x = u32::from(game.marker().pos.x);
        let past_column = u32::from(COLUMN) + 1 - start_x;

        let raised = draw_up_then_right(&mut game, past_column);

        assert_eq!(raised, [Event::Death(DeathCause::Qix)], "{penned_first}");
        let penned = &game.qix()[usize::from(!penned_first)];
        assert!(penned.a().y < f32::from(POCKET_ROWS), "{penned:?} left");
    }
}

#[test]
fn a_stix_closing_across_a_qix_kills_instead_of_claiming() {
    let mut game = closing_through_a_qix();
    let (board, start) = (game.board().clone(), game.marker());
    let up = drawing(Dir::Up, DrawKind::Fast);
    let before_close = (CLOSING_STEPS - 1) * MARKER_STEP_TICKS;
    assert_eq!(deaths(&mut game, up, before_close), []);
    let head = Point::new(W / 2, H - (CLOSING_STEPS as u16 - 1));
    assert_eq!(game.stix().map(Stix::head), Some(head));

    let raised = deaths(&mut game, up, MARKER_STEP_TICKS);

    assert_eq!(raised, [Event::Death(DeathCause::Qix)]);
    assert_eq!(game.board(), &board, "the stix claimed cells");
    assert_eq!(game.marker(), start);
    assert_eq!(game.stix(), None);
    assert_eq!(
        game.qix()[0].a().y,
        f32::from(CLOSING_CHANNEL.1) + CHANNEL_DEPTH
    );
}
