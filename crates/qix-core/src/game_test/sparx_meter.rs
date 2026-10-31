//! The Sparx meter releasing Sparx and turning them Super through
//! [`Game::tick`].

use super::{SEED, hold, new_game};
use crate::qix_test::qix_at;
use crate::tuning::{EXTRA_SPARX, SPARX_METER_TICKS, SPARX_STEP_TICKS};
use crate::{
    Board, DeathCause, Dir, Event, Game, InputState, MeterExpiry, Point, Rng, Sparx, SparxKind,
    SparxMeter,
};

/// Dimensions of the board: the marker starts at `(W / 2, H)`.
const W: u16 = 8;
const H: u16 = 3;

/// Ticks of each countdown of the meter of [`meter_game`].
const LENGTH: u32 = 2;

/// Lattice steps between the marker's start and the Sparx of
/// [`meter_game`] heading for it.
const GAP: u16 = 2;

/// The tick, from one, the Sparx of [`meter_game`] reaches the standing
/// marker: after both expiries of a meter of [`LENGTH`].
const DEATH_TICK: u32 = SPARX_STEP_TICKS * GAP as u32;

/// Where the marker starts, and where the Sparx of [`meter_game`] kills
/// it on [`DEATH_TICK`].
const MARKER_START: Point = Point::new(W / 2, H);

/// The Sparx [`GAP`] steps right of the marker, heading for it.
fn killer() -> Sparx {
    Sparx {
        pos: Point::new(W / 2 + GAP, H),
        dir: Dir::Left,
        kind: SparxKind::Normal,
    }
}

/// A game on a free `W x H` board with a meter of `length`, a short Qix
/// in the top-left cell (the marker never draws, so the Qix never kills)
/// and `sparx` as the level's Sparx.
fn game_with(sparx: Vec<Sparx>, length: u32) -> Game {
    let board = Board::new(W, H);
    let mut game = Game::with_board(board, vec![qix_at((0, 0))], sparx, Rng::new(SEED));
    game.set_sparx_meter(SparxMeter::new(length));
    game
}

/// A game of [`game_with`] whose only level Sparx is the [`killer`].
fn meter_game(length: u32) -> Game {
    game_with(vec![killer()], length)
}

fn stand(game: &mut Game, ticks: u32) {
    hold(game, InputState::default(), ticks);
}

fn kinds(game: &Game) -> Vec<SparxKind> {
    game.sparx().iter().map(|sparx| sparx.kind).collect()
}

#[test]
fn a_new_game_starts_with_a_full_meter() {
    assert_eq!(new_game().sparx_meter(), SparxMeter::new(SPARX_METER_TICKS));
}

#[test]
fn the_meter_counts_down_every_tick() {
    let mut game = new_game();

    stand(&mut game, 1);

    assert_eq!(game.sparx_meter().remaining(), SPARX_METER_TICKS - 1);
}

#[test]
fn nothing_is_released_before_the_first_expiry() {
    let mut game = meter_game(LENGTH);

    stand(&mut game, LENGTH - 1);

    assert_eq!(game.sparx().len(), 1);
}

#[test]
fn the_first_expiry_releases_the_extra_sparx_at_the_top_centre() {
    let mut game = meter_game(LENGTH);

    stand(&mut game, LENGTH);

    let extra = &game.sparx()[1..];
    assert_eq!(extra.len(), EXTRA_SPARX);
    let top_centre = Point::new(W / 2, 0);
    assert!(extra.iter().all(|sparx| sparx.pos == top_centre));
    assert!(extra.iter().all(|sparx| sparx.kind == SparxKind::Normal));
    let dirs: Vec<_> = extra.iter().map(|sparx| sparx.dir).collect();
    assert_eq!(dirs, [Dir::Left, Dir::Right]);
    assert_eq!(game.sparx_meter().next_expiry(), Some(MeterExpiry::Super));
}

#[test]
fn the_second_expiry_turns_every_sparx_super() {
    let mut game = meter_game(LENGTH);
    stand(&mut game, 2 * LENGTH - 1);
    assert!(kinds(&game).iter().all(|&kind| kind == SparxKind::Normal));

    stand(&mut game, 1);

    assert_eq!(kinds(&game), [SparxKind::Super; 1 + EXTRA_SPARX]);
    assert_eq!(game.sparx_meter().next_expiry(), None);
}

#[test]
fn a_death_resets_the_meter_and_undoes_its_expiries() {
    let mut game = meter_game(LENGTH);
    stand(&mut game, DEATH_TICK - 1);
    assert_eq!(kinds(&game), [SparxKind::Super; 1 + EXTRA_SPARX]);

    stand(&mut game, 1);

    assert_eq!(game.events(), [Event::Death(DeathCause::Sparx)]);
    assert_eq!(game.sparx_meter(), SparxMeter::new(LENGTH));
    assert_eq!(kinds(&game), [SparxKind::Normal]);
    assert_eq!(game.sparx()[0].pos, MARKER_START);
}

#[test]
fn a_death_gives_every_level_sparx_back_its_own_starting_kind() {
    let far = Sparx {
        pos: Point::new(0, 0),
        dir: Dir::Down,
        kind: SparxKind::Super,
    };
    let mut game = game_with(vec![killer(), far], LENGTH);
    stand(&mut game, DEATH_TICK - 1);
    assert_eq!(kinds(&game), [SparxKind::Super; 2 + EXTRA_SPARX]);

    stand(&mut game, 1);

    assert_eq!(game.events(), [Event::Death(DeathCause::Sparx)]);
    assert_eq!(kinds(&game), [SparxKind::Normal, SparxKind::Super]);
    assert_eq!(game.sparx()[0].pos, MARKER_START);
    // Down the left border, one step per step of the killer.
    assert_eq!(game.sparx()[1].pos, Point::new(0, GAP));
}

#[test]
fn a_death_before_the_first_expiry_fills_the_meter() {
    let length = 2 * DEATH_TICK;
    let mut game = meter_game(length);

    stand(&mut game, DEATH_TICK);

    assert_eq!(game.events(), [Event::Death(DeathCause::Sparx)]);
    assert_eq!(game.sparx_meter(), SparxMeter::new(length));
}

#[test]
fn the_meter_runs_again_after_a_death() {
    let mut game = meter_game(LENGTH);
    stand(&mut game, DEATH_TICK);

    stand(&mut game, LENGTH);

    assert_eq!(game.sparx().len(), 1 + EXTRA_SPARX);
}
