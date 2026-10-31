use qix_core::{Board, Cell, DrawKind, Game, InputState};

use crate::render::overlay::text;

#[test]
fn a_fresh_game_shows_nothing_claimed_at_tick_zero() {
    assert_eq!(text(&Game::new()), "claimed 0.0%\ntick 0");
}

#[test]
fn the_overlay_follows_the_claimed_share_and_the_ticks() {
    let mut board = Board::new(4, 2);
    board.set_cell(0, 0, Cell::Claimed(DrawKind::Fast));
    let mut game = Game::with_board(board);
    let ticks = 3;
    for _ in 0..ticks {
        game.tick(&InputState::default());
    }
    assert_eq!(text(&game), "claimed 12.5%\ntick 3");
}
