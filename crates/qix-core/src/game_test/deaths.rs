//! Lives, respawning and game over through [`Game::tick`].

use super::qix_hits::{COLUMN, STEPS_UP, band_and_column};
use super::{SEED, draw, drawing, hold, holding, new_game, penned, walk};
use crate::qix_test::qix_at;
use crate::tuning::{
    FUSE_DELAY_TICKS, FUSE_STEP_TICKS, MARKER_STEP_TICKS, RESPAWN_GRACE_TICKS, SPARX_STEP_TICKS,
    STARTING_LIVES,
};
use crate::{
    Board, DeathCause, Dir, DrawKind, Event, Game, InputState, MarkerState, Point, Rng, Sparx,
    SparxKind,
};

/// Dimensions of the boards: the marker starts at `(W / 2, H)`.
const W: u16 = 8;
const H: u16 = 6;

/// Height of the board the Sparx laps, so that a lap is shorter than the
/// grace period but two are longer.
const LAP_H: u16 = 3;

/// Ticks the Sparx takes to walk once round the free `W x LAP_H` board.
const LAP_TICKS: u32 = 2 * (W as u32 + LAP_H as u32) * SPARX_STEP_TICKS;

/// Lattice steps between the marker's start and the Sparx of
/// [`sparx_game`].
const GAP: u16 = 2;

/// Steps of the stix [`die_by_fuse`] draws up from the border.
const FUSE_STIX_STEPS: u32 = 2;

/// A game on a `W x H` board with the Qix penned in its left column, out
/// of reach of the stix, and no Sparx.
fn fuse_game() -> Game {
    let (board, qix) = penned(W, H, 0);
    Game::with_board(board, qix, Vec::new(), Rng::new(SEED))
}

/// A game on a free `W x LAP_H` board with a Sparx [`GAP`] steps right of
/// the marker heading for it, and a short Qix in the top-left cell: the
/// marker never draws, so the Qix never kills.
fn sparx_game() -> Game {
    let sparx = Sparx {
        pos: Point::new(W / 2 + GAP, LAP_H),
        dir: Dir::Left,
        kind: SparxKind::Normal,
    };
    let board = Board::new(W, LAP_H);
    Game::with_board(board, qix_at((0, 0)), vec![sparx], Rng::new(SEED))
}

/// Ticks the fuse of a stix of `segments` segments takes to light and
/// burn to a marker standing still at its head.
fn burn_ticks(segments: u32) -> u32 {
    FUSE_DELAY_TICKS + FUSE_STEP_TICKS * segments
}

/// Ticks `game` `ticks` times with nothing held.
fn stand(game: &mut Game, ticks: u32) {
    hold(game, InputState::default(), ticks);
}

/// Waits out the grace period, draws a stix [`FUSE_STIX_STEPS`] up from
/// the marker and stands until the fuse kills it.
fn die_by_fuse(game: &mut Game) {
    stand(game, game.grace_ticks());
    draw(game, Dir::Up, DrawKind::Fast, FUSE_STIX_STEPS);
    stand(game, burn_ticks(1));
}

/// Kills the marker of a [`fuse_game`] by the fuse, then stands and holds
/// the draw button up so that the marker tries its first step of a stix
/// `tick` ticks after the death.
fn try_drawing_at(tick: u32) -> Game {
    let mut game = fuse_game();
    die_by_fuse(&mut game);
    stand(&mut game, tick - MARKER_STEP_TICKS);
    hold(
        &mut game,
        drawing(Dir::Up, DrawKind::Fast),
        MARKER_STEP_TICKS,
    );
    game
}

/// Ticks `game` `ticks` times with nothing held and returns the tick,
/// from one, of each death raised.
fn death_ticks(game: &mut Game, ticks: u32) -> Vec<u32> {
    (1..=ticks)
        .filter(|_| {
            game.tick(&InputState::default());
            game.events().iter().any(|e| matches!(e, Event::Death(_)))
        })
        .collect()
}

#[test]
fn a_new_game_has_every_life() {
    let game = new_game();

    assert_eq!(game.lives(), STARTING_LIVES);
    assert!(!game.is_over());
}

#[test]
fn a_fuse_death_takes_a_life() {
    let mut game = fuse_game();

    die_by_fuse(&mut game);

    assert_eq!(game.events(), [Event::Death(DeathCause::Fuse)]);
    assert_eq!(game.lives(), STARTING_LIVES - 1);
}

#[test]
fn a_sparx_death_takes_a_life() {
    let mut game = sparx_game();

    stand(&mut game, SPARX_STEP_TICKS * u32::from(GAP));

    assert_eq!(game.events(), [Event::Death(DeathCause::Sparx)]);
    assert_eq!(game.lives(), STARTING_LIVES - 1);
}

#[test]
fn a_qix_death_takes_a_life_and_the_marker_back_to_the_stix_start() {
    let mut game = band_and_column();
    let start = game.marker().pos;
    draw(&mut game, Dir::Up, DrawKind::Fast, STEPS_UP);

    let past_column = u32::from(COLUMN - start.x) + 1;
    let ticks = (1..=past_column * MARKER_STEP_TICKS).find(|_| {
        game.tick(&drawing(Dir::Right, DrawKind::Fast));
        !game.events().is_empty()
    });

    assert!(ticks.is_some(), "the Qix never killed");
    assert_eq!(game.events(), [Event::Death(DeathCause::Qix)]);
    assert_eq!(game.lives(), STARTING_LIVES - 1);
    assert_eq!(game.marker().pos, start);
}

#[test]
fn a_death_while_drawing_clears_the_stix_and_respawns_at_its_start() {
    let mut game = fuse_game();
    walk(&mut game, Dir::Right, 1);
    let start = game.marker().pos;
    draw(&mut game, Dir::Up, DrawKind::Fast, FUSE_STIX_STEPS);
    draw(&mut game, Dir::Right, DrawKind::Fast, 1);

    stand(&mut game, burn_ticks(2));

    assert_eq!(game.events(), [Event::Death(DeathCause::Fuse)]);
    assert_eq!(game.marker().pos, start);
    assert_eq!(game.marker().state, MarkerState::OnBorder);
    assert_eq!(game.stix(), None);
    assert_eq!(game.fuse(), None);
}

#[test]
fn a_death_on_the_border_leaves_the_marker_where_it_is() {
    let mut game = sparx_game();
    let start = game.marker().pos;

    stand(&mut game, SPARX_STEP_TICKS * u32::from(GAP));

    assert_eq!(game.events(), [Event::Death(DeathCause::Sparx)]);
    assert_eq!(game.marker().pos, start);
    assert_eq!(game.marker().state, MarkerState::OnBorder);
}

#[test]
fn a_sparx_cannot_kill_during_the_grace_period() {
    assert!((LAP_TICKS..2 * LAP_TICKS).contains(&RESPAWN_GRACE_TICKS));
    let mut game = sparx_game();
    stand(&mut game, SPARX_STEP_TICKS * u32::from(GAP));
    assert_eq!(game.lives(), STARTING_LIVES - 1);

    let deaths = death_ticks(&mut game, LAP_TICKS);
    assert_eq!(game.sparx()[0].pos, game.marker().pos, "no lap");
    assert_eq!(deaths, []);

    let deaths = death_ticks(&mut game, LAP_TICKS);
    assert_eq!(deaths, [LAP_TICKS], "the grace period did not end");
}

#[test]
fn the_marker_cannot_draw_during_the_grace_period() {
    let mut game = fuse_game();
    die_by_fuse(&mut game);
    let start = game.marker().pos;

    for _ in 0..RESPAWN_GRACE_TICKS {
        game.tick(&drawing(Dir::Up, DrawKind::Fast));
        assert_eq!(game.marker().pos, start);
        assert_eq!(game.stix(), None);
    }
    hold(
        &mut game,
        drawing(Dir::Up, DrawKind::Fast),
        MARKER_STEP_TICKS,
    );

    assert_eq!(game.marker().state, MarkerState::Drawing);
}

#[test]
fn the_grace_period_ends_after_its_last_tick() {
    let last = try_drawing_at(RESPAWN_GRACE_TICKS);
    assert_eq!(last.grace_ticks(), 0);
    assert_eq!(
        last.marker().state,
        MarkerState::OnBorder,
        "drew on the last tick of the grace period"
    );

    let after = try_drawing_at(RESPAWN_GRACE_TICKS + 1);
    assert_eq!(
        after.marker().state,
        MarkerState::Drawing,
        "could not draw on the first tick after the grace period"
    );
}

#[test]
fn the_marker_walks_the_border_during_the_grace_period() {
    let mut game = fuse_game();
    die_by_fuse(&mut game);
    let start = game.marker().pos;

    walk(&mut game, Dir::Right, 1);

    assert!(game.grace_ticks() > 0, "the grace period ended");
    assert_eq!(game.marker().pos, Point::new(start.x + 1, start.y));
}

#[test]
fn losing_the_last_life_raises_game_over() {
    let mut game = fuse_game();
    for lives in (1..STARTING_LIVES).rev() {
        die_by_fuse(&mut game);
        assert_eq!(game.events(), [Event::Death(DeathCause::Fuse)]);
        assert_eq!(game.lives(), lives);
    }

    die_by_fuse(&mut game);

    let game_over = [Event::Death(DeathCause::Fuse), Event::GameOver];
    assert_eq!(game.events(), game_over);
    assert_eq!(game.lives(), 0);
    assert!(game.is_over());
    assert_eq!(game.grace_ticks(), 0, "a grace period after game over");
}

#[test]
fn a_game_over_no_longer_ticks() {
    let mut game = fuse_game();
    for _ in 0..STARTING_LIVES {
        die_by_fuse(&mut game);
    }
    let (ticks, marker, qix) = (game.ticks(), game.marker(), game.qix().clone());

    hold(&mut game, holding(Dir::Right), MARKER_STEP_TICKS);

    assert_eq!(game.ticks(), ticks);
    assert_eq!(game.events(), []);
    assert_eq!(game.marker(), marker);
    assert_eq!(game.qix(), &qix);
}
