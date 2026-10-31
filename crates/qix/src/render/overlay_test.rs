use qix_core::{Board, Cell, DrawKind, Game, InputState, Qix, Rng, Vec2};

use crate::render::overlay::text;

/// Seed of every game the tests play.
const SEED: u64 = 1;

#[test]
fn a_fresh_game_shows_nothing_claimed_at_tick_zero() {
    assert_eq!(text(&Game::new(Rng::new(SEED))), "claimed 0.0%\ntick 0");
}

#[test]
fn the_overlay_follows_the_claimed_share_and_the_ticks() {
    let mut board = Board::new(4, 2);
    board.set_cell(0, 0, Cell::Claimed(DrawKind::Fast));
    // A short Qix standing in the free bottom-right cell.
    let qix = Qix::new(
        Vec2::new(3.25, 1.5),
        Vec2::new(3.75, 1.5),
        Vec2::default(),
        0.0,
    );
    let mut game = Game::with_board(board, qix, Vec::new(), Rng::new(SEED));
    let ticks = 3;
    for _ in 0..ticks {
        game.tick(&InputState::default());
    }
    assert_eq!(text(&game), "claimed 12.5%\ntick 3");
}
