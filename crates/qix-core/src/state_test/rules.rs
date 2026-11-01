//! [`Session::set_rules`]: the title screen's game restarts with the
//! new rules, a game in play keeps its own until the next one.

use super::{CONFIRM, SEED, game_over, level_game, playing};
use crate::{ClaimThreshold, Game, GameState, Rng, Rules, Session, StartingLives};

/// The claim threshold and the starting lives of [`rules`].
const RULES_THRESHOLD: f32 = 0.55;
const RULES_LIVES: u32 = 7;

/// Rules other than the default ones.
fn rules() -> Rules {
    let rules = Rules {
        threshold: ClaimThreshold::new(RULES_THRESHOLD),
        starting_lives: StartingLives::new(RULES_LIVES),
    };
    assert_ne!(rules, Rules::default());
    rules
}

#[test]
fn new_rules_on_the_title_screen_restart_its_game_with_them() {
    let mut session = Session::new(Game::new(Rng::new(SEED)));

    session.set_rules(rules());

    assert_eq!(session.state(), GameState::Title);
    let game = session.game();
    assert_eq!(game.lives(), RULES_LIVES);
    assert_eq!(game.claim_threshold(), rules().threshold);
    assert_eq!(game.ticks(), 0);
}

#[test]
fn the_game_started_after_new_rules_plays_by_them() {
    let mut session = Session::new(Game::new(Rng::new(SEED)));
    session.set_rules(rules());

    session.tick(&CONFIRM);

    assert_eq!(session.state(), GameState::Playing);
    assert_eq!(session.game().lives(), RULES_LIVES);
}

#[test]
fn a_game_in_play_keeps_its_own_rules() {
    let mut session = playing(level_game());
    let (lives, threshold) = (session.game().lives(), session.game().claim_threshold());

    session.set_rules(rules());

    assert_eq!(session.state(), GameState::Playing);
    assert_eq!(session.game().lives(), lives);
    assert_eq!(session.game().claim_threshold(), threshold);
}

#[test]
fn the_game_after_game_over_plays_by_the_new_rules() {
    let (mut session, _) = game_over();
    session.set_rules(rules());

    session.tick(&CONFIRM);

    assert_eq!(session.state(), GameState::Title);
    assert_eq!(session.game().lives(), RULES_LIVES);
    assert_eq!(session.game().claim_threshold(), rules().threshold);
}
