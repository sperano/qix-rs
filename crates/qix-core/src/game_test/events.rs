//! The [`Event::Draw`] and [`Event::Claim`] a stix raises through
//! [`Game::tick`], which the application plays sounds for.

use super::{SEED, draw_step_ticks, drawing, hold, holding, pen};
use crate::tuning::{BOARD_H, BOARD_W, MARKER_STEP_TICKS};
use crate::{Board, Dir, DrawKind, Event, Game, Rng};

/// The column of the full board the Qix is [`pen`]ned in, left of the
/// stix the tests draw up from the marker's start.
const PEN_COLUMN: u16 = 1;

/// A full-size game with no Sparx and its Qix penned in [`PEN_COLUMN`].
fn penned_game() -> Game {
    let mut board = Board::new(BOARD_W, BOARD_H);
    let qix = pen(&mut board, PEN_COLUMN);
    Game::with_board(board, vec![qix], Vec::new(), Rng::new(SEED))
}

/// Ticks `game` with `kind` held up for `ticks` ticks and returns every
/// event raised, in order.
fn events_drawing_up(game: &mut Game, kind: DrawKind, ticks: u32) -> Vec<Event> {
    let mut events = Vec::new();
    for _ in 0..ticks {
        game.tick(&drawing(Dir::Up, kind));
        events.extend_from_slice(game.events());
    }
    events
}

#[test]
fn each_step_of_the_stix_raises_a_draw_of_its_kind() {
    for kind in [DrawKind::Fast, DrawKind::Slow] {
        let mut game = penned_game();
        let steps = 3;

        let events = events_drawing_up(&mut game, kind, steps * draw_step_ticks(kind));

        assert_eq!(events, vec![Event::Draw(kind); steps as usize], "{kind:?}");
    }
}

#[test]
fn only_the_tick_of_a_step_raises_its_draw() {
    let mut game = penned_game();
    let input = drawing(Dir::Up, DrawKind::Slow);

    hold(&mut game, input, draw_step_ticks(DrawKind::Slow) - 1);
    assert_eq!(game.events(), []);
    game.tick(&input);
    assert_eq!(game.events(), [Event::Draw(DrawKind::Slow)]);
}

#[test]
fn walking_the_border_raises_no_draw() {
    let mut game = penned_game();

    for _ in 0..MARKER_STEP_TICKS * 3 {
        game.tick(&holding(Dir::Right));
        assert_eq!(game.events(), []);
    }
}

#[test]
fn the_closing_step_raises_its_draw_then_the_claim() {
    let mut game = penned_game();
    let ticks = u32::from(BOARD_H) * draw_step_ticks(DrawKind::Fast);
    let before = game.board().claimed_fraction();

    let events = events_drawing_up(&mut game, DrawKind::Fast, ticks);

    assert_eq!(game.stix(), None, "the stix did not close");
    let claim = events
        .iter()
        .position(|event| matches!(event, Event::Claim { .. }))
        .expect("no claim");
    assert_eq!(events[claim - 1], Event::Draw(DrawKind::Fast));
    let Event::Claim { cells, kind } = events[claim] else {
        unreachable!()
    };
    assert_eq!(kind, DrawKind::Fast);
    let cells_after = game.board().claimed_fraction() - before;
    let total = f32::from(BOARD_W) * f32::from(BOARD_H);
    assert_eq!(cells as f32, (cells_after * total).round());
    let after_claim = &events[claim + 1..];
    assert!(
        after_claim.iter().all(|event| *event == Event::ExtraLife),
        "{after_claim:?}"
    );
}
