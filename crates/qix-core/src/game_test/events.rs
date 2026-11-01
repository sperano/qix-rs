//! The [`Event::Draw`] and [`Event::Claim`] a stix raises through
//! [`Game::tick`], which the application plays sounds for.

use super::{SEED, draw, draw_step_ticks, drawing, hold, holding, pen};
use crate::tuning::{BOARD_H, BOARD_W, MARKER_STEP_TICKS, STARTING_LIVES};
use crate::{Board, Dir, DrawKind, Event, Game, Rng};

/// The column of the full board the Qix is [`pen`]ned in, left of the
/// stix the tests draw up from the marker's start.
const PEN_COLUMN: u16 = 1;

/// Steps of the first segment of the stix
/// [`the_closing_step_raises_its_draw_then_the_claim_then_its_extra_lives`]
/// draws with another kind than the rest.
const FIRST_SEGMENT_STEPS: u32 = 3;

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

/// Draws the stix up from where the marker is with `kind` until it
/// closes, and returns the events of the closing tick.
fn close_up(game: &mut Game, kind: DrawKind) -> Vec<Event> {
    let input = drawing(Dir::Up, kind);
    let mut drawn = game.stix().is_some();
    for _ in 0..u32::from(BOARD_H) * draw_step_ticks(kind) {
        game.tick(&input);
        if drawn && game.stix().is_none() {
            return game.events().to_vec();
        }
        drawn |= game.stix().is_some();
    }
    panic!("the stix did not close");
}

/// The cells claimed on `game`'s board since it had `before` of it.
fn cells_claimed_since(game: &Game, before: f32) -> usize {
    let total = f32::from(BOARD_W) * f32::from(BOARD_H);
    ((game.board().claimed_fraction() - before) * total).round() as usize
}

#[test]
fn the_closing_step_raises_its_draw_then_the_claim_then_its_extra_lives() {
    let cases = [
        (None, DrawKind::Fast, DrawKind::Fast),
        (Some(DrawKind::Slow), DrawKind::Slow, DrawKind::Slow),
        (Some(DrawKind::Slow), DrawKind::Fast, DrawKind::Fast),
        (Some(DrawKind::Fast), DrawKind::Slow, DrawKind::Fast),
    ];
    for (first, last, claim_kind) in cases {
        let mut game = penned_game();
        let before = game.board().claimed_fraction();
        if let Some(kind) = first {
            draw(&mut game, Dir::Up, kind, FIRST_SEGMENT_STEPS);
        }

        let events = close_up(&mut game, last);

        let lives = game.lives() - STARTING_LIVES;
        assert!(lives > 0, "{first:?} then {last:?}: no extra life");
        let mut expected = vec![
            Event::Draw(last),
            Event::Claim {
                cells: cells_claimed_since(&game, before),
                kind: claim_kind,
            },
        ];
        expected.extend(vec![Event::ExtraLife; lives as usize]);
        assert_eq!(events, expected, "{first:?} then {last:?}");
    }
}
