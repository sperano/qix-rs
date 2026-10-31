use qix_core::tuning::SPARX_STEP_TICKS;
use qix_core::{
    Board, Cell, Dir, DrawKind, Event, Game, InputState, Point, Qix, Rng, Sparx, SparxKind, Vec2,
};

use crate::render::overlay::text;

/// Seed of every game the tests play.
const SEED: u64 = 1;

#[test]
fn a_fresh_game_shows_nothing_claimed_and_every_life_at_tick_zero() {
    assert_eq!(
        text(&Game::new(Rng::new(SEED))),
        "claimed 0.0%\nlives 3\ntick 0"
    );
}

/// A short Qix standing in the free bottom-right cell of a 4 x 2 board.
fn corner_qix() -> Qix {
    Qix::new(
        Vec2::new(3.25, 1.5),
        Vec2::new(3.75, 1.5),
        Vec2::default(),
        0.0,
    )
}

#[test]
fn the_overlay_follows_the_claimed_share_and_the_ticks() {
    let mut board = Board::new(4, 2);
    board.set_cell(0, 0, Cell::Claimed(DrawKind::Fast));
    let mut game = Game::with_board(board, vec![corner_qix()], Vec::new(), Rng::new(SEED));
    let ticks = 3;
    for _ in 0..ticks {
        game.tick(&InputState::default());
    }
    assert_eq!(text(&game), "claimed 12.5%\nlives 3\ntick 3");
}

#[test]
fn the_overlay_shows_a_life_lost() {
    // A Sparx one step right of the marker's start, (2, 2), heading for it.
    let sparx = Sparx {
        pos: Point::new(3, 2),
        dir: Dir::Left,
        kind: SparxKind::Normal,
    };
    let mut game = Game::with_board(
        Board::new(4, 2),
        vec![corner_qix()],
        vec![sparx],
        Rng::new(SEED),
    );
    for _ in 0..SPARX_STEP_TICKS {
        game.tick(&InputState::default());
    }
    assert!(matches!(game.events(), [Event::Death(_)]), "no death");
    assert_eq!(
        text(&game),
        format!("claimed 0.0%\nlives 2\ntick {SPARX_STEP_TICKS}")
    );
}
