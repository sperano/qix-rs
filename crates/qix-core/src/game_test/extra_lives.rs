//! Extra lives through [`Game::tick`]: one for every multiple of
//! [`EXTRA_LIFE_EVERY`] the score crosses, up to [`MAX_LIVES`].

use super::{SEED, draw, pen, walk};
use crate::tuning::{
    BOARD_H, BOARD_W, CLAIM_THRESHOLD_RANGE, EXTRA_LIFE_EVERY, MAX_LIVES, MAX_MULTIPLIER,
    STARTING_LIVES,
};
use crate::{Board, ClaimThreshold, Dir, DrawKind, Event, Game, Rng};

/// The column of the full board the Qix is [`pen`]ned in, left of every
/// stix the tests draw: each claims the board right of its stix.
const PEN_COLUMN: u16 = 1;

/// Lattice steps left of the marker's start the stix of
/// [`the_level_bonus_counts_towards_extra_lives`] runs up: far enough
/// that its claim reaches the lowest threshold and scores a bonus.
const BONUS_STIX_OFFSET: u32 = 40;

/// A full-size game with no Sparx and its Qix penned in [`PEN_COLUMN`].
fn penned_game() -> Game {
    let mut board = Board::new(BOARD_W, BOARD_H);
    let qix = pen(&mut board, PEN_COLUMN);
    Game::with_board(board, vec![qix], Vec::new(), Rng::new(SEED))
}

/// Draws the stix up from the marker to the top edge with `kind`, and
/// checks that it closed.
fn close_up(game: &mut Game, kind: DrawKind) {
    draw(game, Dir::Up, kind, u32::from(BOARD_H));
    assert_eq!(game.stix(), None, "the stix did not close");
}

/// Draws the stix down from the marker on the top edge to the bottom
/// edge with `kind`, and checks that it closed.
fn close_down(game: &mut Game, kind: DrawKind) {
    draw(game, Dir::Down, kind, u32::from(BOARD_H));
    assert_eq!(game.stix(), None, "the stix did not close");
}

/// The [`Event::ExtraLife`] the last tick raised.
fn extra_life_events(game: &Game) -> u32 {
    let count = game
        .events()
        .iter()
        .filter(|&&event| event == Event::ExtraLife)
        .count();
    u32::try_from(count).unwrap()
}

/// The multiples of [`EXTRA_LIFE_EVERY`] up to `points`.
fn steps(points: u32) -> u32 {
    points / EXTRA_LIFE_EVERY
}

#[test]
fn a_claim_crossing_several_steps_earns_a_life_for_each() {
    let mut game = penned_game();

    close_up(&mut game, DrawKind::Fast);

    let crossed = steps(game.score().points());
    assert!(crossed > 1, "the claim crossed {crossed} steps");
    assert!(STARTING_LIVES + crossed <= MAX_LIVES, "the cap was reached");
    assert_eq!(game.lives(), STARTING_LIVES + crossed);
    assert_eq!(extra_life_events(&game), crossed);
}

#[test]
fn extra_lives_stop_at_the_maximum() {
    let mut game = penned_game();
    game.set_multiplier(MAX_MULTIPLIER);

    close_up(&mut game, DrawKind::Slow);

    let crossed = steps(game.score().points());
    assert!(
        STARTING_LIVES + crossed > MAX_LIVES,
        "the cap was not reached"
    );
    assert_eq!(game.lives(), MAX_LIVES);
    assert_eq!(extra_life_events(&game), MAX_LIVES - STARTING_LIVES);
}

#[test]
fn a_claim_at_the_maximum_lives_earns_none() {
    let mut game = penned_game();
    game.set_multiplier(MAX_MULTIPLIER);
    close_up(&mut game, DrawKind::Slow);
    let score = game.score().points();

    walk(&mut game, Dir::Left, 1);
    close_down(&mut game, DrawKind::Slow);

    assert!(
        steps(game.score().points()) > steps(score),
        "no step crossed"
    );
    assert_eq!(game.lives(), MAX_LIVES);
    assert_eq!(extra_life_events(&game), 0);
}

#[test]
fn the_level_bonus_counts_towards_extra_lives() {
    let mut game = penned_game();
    game.set_claim_threshold(ClaimThreshold::new(*CLAIM_THRESHOLD_RANGE.start()));
    walk(&mut game, Dir::Left, BONUS_STIX_OFFSET);

    close_up(&mut game, DrawKind::Fast);

    let Some(&Event::LevelComplete { bonus, .. }) = game.events().last() else {
        panic!("the level did not end: {:?}", game.events());
    };
    let score = game.score().points();
    assert!(
        steps(score - bonus) < steps(score),
        "the bonus crossed no step"
    );
    assert_eq!(game.lives(), STARTING_LIVES + steps(score));
    assert_eq!(extra_life_events(&game), steps(score));
}
