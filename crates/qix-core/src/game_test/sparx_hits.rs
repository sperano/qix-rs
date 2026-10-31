//! The Sparx walking the border and killing the marker through
//! [`Game::tick`].

use super::{SEED, draw, holding, new_game, penned};
use crate::tuning::{BOARD_W, MARKER_STEP_TICKS, SPARX_STEP_TICKS};
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

/// Ticks a Sparx takes to walk [`GAP`] steps.
const GAP_TICKS: u32 = SPARX_STEP_TICKS * GAP as u32;

/// A tick on which the marker and the Sparx both step.
const BOTH_STEP_TICK: u32 = MARKER_STEP_TICKS * SPARX_STEP_TICKS;

/// Height of the board of [`a_sparx_cut_off_by_a_claim_snaps_back_to_the_border`],
/// tall enough to pen its Qix in [`CUT_COLUMN`], right of the stix.
const CUT_H: u16 = 6;
const CUT_COLUMN: u16 = W - 2;

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

/// Ticks `game` `ticks` times with `input` and returns the tick, from
/// one, of each Sparx death raised.
fn sparx_death_ticks(game: &mut Game, input: InputState, ticks: u32) -> Vec<u32> {
    (1..=ticks)
        .filter(|_| {
            game.tick(&input);
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

    for tick in 1..=GAP_TICKS {
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

    let deaths = sparx_death_ticks(&mut game, InputState::default(), GAP_TICKS);

    assert_eq!(game.sparx()[0].pos, game.marker().pos);
    assert_eq!(deaths, [GAP_TICKS]);
}

#[test]
fn a_sparx_walking_away_from_the_marker_does_not_kill() {
    let start = Point::new(W / 2 + 1, H);
    let mut game = sparx_game(vec![sparx(start, Dir::Right)]);

    let deaths = sparx_death_ticks(&mut game, InputState::default(), GAP_TICKS);

    assert_eq!(deaths, []);
}

#[test]
fn a_marker_and_a_sparx_swapping_points_kills() {
    // Steps each takes before `BOTH_STEP_TICK`, after which the marker,
    // heading right from (W / 2, H), and the Sparx, heading left, are on
    // neighbouring points and swap them.
    let steps = |every: u32| u16::try_from(BOTH_STEP_TICK / every - 1).unwrap();
    let marker_before = W / 2 + steps(MARKER_STEP_TICKS);
    let start = Point::new(marker_before + 1 + steps(SPARX_STEP_TICKS), H);
    let mut game = sparx_game(vec![sparx(start, Dir::Left)]);

    let deaths = sparx_death_ticks(&mut game, holding(Dir::Right), BOTH_STEP_TICK);

    assert_eq!(game.marker().pos, Point::new(marker_before + 1, H));
    assert_eq!(game.sparx()[0].pos, Point::new(marker_before, H));
    assert_eq!(deaths, [BOTH_STEP_TICK]);
}

#[test]
fn a_sparx_cut_off_by_a_claim_snaps_back_to_the_border() {
    // The stix goes straight up from (W / 2, CUT_H) and claims the left
    // half, where the Sparx walks.
    let (board, qix) = penned(W, CUT_H, CUT_COLUMN);
    let walker = sparx(Point::new(0, 1), Dir::Down);
    let mut game = Game::with_board(board, qix, vec![walker], Rng::new(SEED));

    draw(&mut game, Dir::Up, DrawKind::Fast, u32::from(CUT_H));

    let sparx = game.sparx()[0];
    assert_eq!(game.marker().state, MarkerState::OnBorder);
    assert!(game.board().cell(0, 0).is_claimed(), "nothing was claimed");
    assert!(game.board().is_border(sparx.pos), "{sparx:?}");
    assert!(sparx.pos.x >= W / 2, "{sparx:?}");
}
