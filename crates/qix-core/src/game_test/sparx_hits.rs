//! The Sparx walking the border and killing the marker through
//! [`Game::tick`].

use super::{SEED, draw, new_game};
use crate::tuning::{BOARD_W, SPARX_STEP_TICKS};
use crate::{
    Board, DeathCause, Dir, DrawKind, Event, Game, InputState, MarkerState, Point, Qix, Rng, Sparx,
    SparxKind, Vec2,
};

/// Dimensions of the boards: the marker starts at `(W / 2, H)`.
const W: u16 = 8;
const H: u16 = 3;

/// Lattice steps between the marker's start and the Sparx of
/// [`a_sparx_walking_into_a_standing_marker_kills`].
const GAP: u16 = 2;

/// Steps of a stix straight up from the marker's start across the board.
const ACROSS: u32 = H as u32;

fn sparx(pos: Point, dir: Dir) -> Sparx {
    Sparx {
        pos,
        dir,
        kind: SparxKind::Normal,
    }
}

/// A game on a free `W x H` board with `sparx` and a short Qix standing
/// in the cell right of the board's centre.
fn sparx_game(sparx: Vec<Sparx>) -> Game {
    let (x, y) = (f32::from(W / 2) + 1.5, f32::from(H / 2) + 0.5);
    let qix = Qix::new(
        Vec2::new(x - 0.25, y),
        Vec2::new(x + 0.25, y),
        Vec2::default(),
        0.0,
    );
    Game::with_board(Board::new(W, H), qix, sparx, Rng::new(SEED))
}

/// Ticks `game` `ticks` times without input and returns the tick, from
/// one, of each Sparx death raised.
fn sparx_death_ticks(game: &mut Game, ticks: u32) -> Vec<u32> {
    (1..=ticks)
        .filter(|_| {
            game.tick(&InputState::default());
            game.events().contains(&Event::Death(DeathCause::Sparx))
        })
        .collect()
}

#[test]
fn a_new_game_spawns_two_sparx_at_the_top_centre_heading_apart() {
    let game = new_game();
    let top_centre = Point::new(BOARD_W / 2, 0);

    let spawned: Vec<(Point, Dir)> = game.sparx().iter().map(|s| (s.pos, s.dir)).collect();

    assert_eq!(spawned, [(top_centre, Dir::Left), (top_centre, Dir::Right)]);
}

#[test]
fn sparx_step_once_every_sparx_step_ticks() {
    let mut game = new_game();
    let top_centre = Point::new(BOARD_W / 2, 0);

    for tick in 1..=SPARX_STEP_TICKS * u32::from(GAP) {
        game.tick(&InputState::default());
        let steps = u16::try_from(tick / SPARX_STEP_TICKS).unwrap();
        let xs: Vec<u16> = game.sparx().iter().map(|s| s.pos.x).collect();
        assert_eq!(xs, [top_centre.x - steps, top_centre.x + steps]);
    }
}

#[test]
fn a_sparx_walking_into_a_standing_marker_kills() {
    let start = Point::new(W / 2 + GAP, H);
    let mut game = sparx_game(vec![sparx(start, Dir::Left)]);

    let deaths = sparx_death_ticks(&mut game, SPARX_STEP_TICKS * u32::from(GAP));

    assert_eq!(game.sparx()[0].pos, game.marker().pos);
    assert_eq!(deaths, [SPARX_STEP_TICKS * u32::from(GAP)]);
}

#[test]
fn a_sparx_walking_away_from_the_marker_does_not_kill() {
    let start = Point::new(W / 2 + 1, H);
    let mut game = sparx_game(vec![sparx(start, Dir::Right)]);

    let deaths = sparx_death_ticks(&mut game, SPARX_STEP_TICKS * u32::from(GAP));

    assert_eq!(deaths, []);
}

#[test]
fn a_sparx_cut_off_by_a_claim_snaps_back_to_the_border() {
    // The stix goes straight up from (W / 2, H) and claims the left half,
    // where the Sparx walks.
    let mut game = sparx_game(vec![sparx(Point::new(0, 1), Dir::Down)]);

    draw(&mut game, Dir::Up, DrawKind::Fast, ACROSS);

    let sparx = game.sparx()[0];
    assert_eq!(game.marker().state, MarkerState::OnBorder);
    assert!(game.board().cell(0, 0).is_claimed(), "nothing was claimed");
    assert!(game.board().is_border(sparx.pos), "{sparx:?}");
    assert!(sparx.pos.x >= W / 2, "{sparx:?}");
}
