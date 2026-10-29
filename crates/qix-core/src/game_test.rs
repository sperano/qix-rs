use crate::{Dir, Game, InputState};

#[test]
fn new_game_has_not_ticked() {
    assert_eq!(Game::new().ticks(), 0);
}

#[test]
fn each_tick_advances_one_step() {
    let mut game = Game::new();
    let input = InputState {
        dir: Some(Dir::Left),
        ..InputState::default()
    };

    game.tick(&input);
    game.tick(&InputState::default());

    assert_eq!(game.ticks(), 2);
}
