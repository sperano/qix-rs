//! Super Sparx chasing the marker along the stix through [`Game::tick`].

use super::qix_hits::{COLUMN, STEPS_UP, band_and_column, band_and_column_with};
use super::{SEED, draw, drawing, hold, penned};
use crate::tuning::{
    FUSE_DELAY_TICKS, FUSE_STEP_TICKS, MARKER_STEP_TICKS, SPARX_STEP_TICKS, SUPER_SPARX_STEP_TICKS,
};
use crate::{DeathCause, Dir, DrawKind, Event, Game, InputState, Point, Rng, Sparx, SparxKind};

/// Dimensions of the board: the marker starts at `(W / 2, H)`.
const W: u16 = 8;
const H: u16 = 6;

/// The column the Qix is penned in, clear of the stix.
const PEN_COLUMN: u16 = 1;

/// Steps of the stix drawn up from the marker's start: it stops short of
/// the top edge, so it stays open.
const STIX_STEPS: u16 = H - 2;

/// Lattice steps between the marker's start and the Sparx heading for it
/// along the bottom edge.
const GAP: u16 = 2;

/// Ticks the tests watch: long enough for a Sparx to reach the stix's
/// start and walk all of it, short of the fuse.
const WATCH_TICKS: u32 =
    (GAP as u32 + 1) * SPARX_STEP_TICKS + (STIX_STEPS as u32 + 1) * SUPER_SPARX_STEP_TICKS;

/// Where the marker starts, and the stix with it.
const START: Point = Point::new(W / 2, H);

/// Where the marker stands at the head of the stix.
const HEAD: Point = Point::new(W / 2, H - STIX_STEPS);

/// Steps of the stix the fuse burns in
/// [`a_sparx_on_the_stix_snaps_to_the_border_on_a_fuse_death`], drawn
/// straight up: two of its lattice points lie between its start and its
/// head.
const FUSE_STIX_STEPS: u32 = 3;

/// Width of that test's board: the bottom edge right of the marker's start
/// is long enough for the Sparx to reach the stix late.
const FUSE_W: u16 = 32;

/// The tick the fuse reaches the head: the stix drawn fast, the fuse's
/// delay, then one fuse step for the stix's one segment.
const FUSE_DEATH_TICK: u32 =
    FUSE_STIX_STEPS * MARKER_STEP_TICKS + FUSE_DELAY_TICKS + FUSE_STEP_TICKS;

/// The tick the Sparx steps onto the stix point after the start: it is on
/// the last point before the head on [`FUSE_DEATH_TICK`], a step short of
/// the marker.
const ONTO_STIX_TICK: u32 = FUSE_DEATH_TICK - (FUSE_STIX_STEPS - 2) * SUPER_SPARX_STEP_TICKS;

/// Lattice steps between the marker's start and the Sparx: it reaches the
/// stix's start on its last border step before [`ONTO_STIX_TICK`].
const FUSE_GAP: u16 = ((ONTO_STIX_TICK - 1) / SPARX_STEP_TICKS) as u16;

/// A Sparx of `kind` [`GAP`] steps right of the marker on the bottom
/// edge, heading for it.
fn sparx(kind: SparxKind) -> Sparx {
    Sparx {
        pos: Point::new(W / 2 + GAP, H),
        dir: Dir::Left,
        kind,
    }
}

/// A game on a [`penned`] board with one Sparx of `kind`.
fn game_with(kind: SparxKind) -> Game {
    let (board, qix) = penned(W, H, PEN_COLUMN);
    Game::with_board(board, vec![qix], vec![sparx(kind)], Rng::new(SEED))
}

/// What one tick of [`watch`] saw.
#[derive(Debug)]
struct Seen {
    /// The tick, from one.
    tick: u32,
    /// Where the Sparx is after it.
    pos: Point,
    /// The Sparx killed the marker on it.
    sparx_death: bool,
}

/// Draws [`STIX_STEPS`] up from [`START`], then stands for the rest of
/// [`WATCH_TICKS`], recording every tick.
fn watch(game: &mut Game) -> Vec<Seen> {
    let input = |tick: u32| {
        let drawing = tick <= u32::from(STIX_STEPS) * SUPER_SPARX_STEP_TICKS;
        InputState {
            dir: drawing.then_some(Dir::Up),
            fast: drawing,
            ..InputState::default()
        }
    };
    (1..=WATCH_TICKS)
        .map(|tick| {
            game.tick(&input(tick));
            Seen {
                tick,
                pos: game.sparx()[0].pos,
                sparx_death: game.events().contains(&Event::Death(DeathCause::Sparx)),
            }
        })
        .collect()
}

/// The points of the stix other than its start the Sparx of `seen`
/// stepped onto, with the tick it did.
fn on_the_stix(seen: &[Seen]) -> Vec<(u32, Point)> {
    let mut steps: Vec<_> = seen
        .iter()
        .filter(|seen| seen.pos.x == START.x && (HEAD.y..START.y).contains(&seen.pos.y))
        .map(|seen| (seen.tick, seen.pos))
        .collect();
    steps.dedup_by_key(|&mut (_, pos)| pos);
    steps
}

#[test]
fn a_super_sparx_enters_the_stix_and_kills_the_marker_at_its_head() {
    let mut game = game_with(SparxKind::Super);

    let seen = watch(&mut game);

    let chase = on_the_stix(&seen);
    let points: Vec<_> = chase.iter().map(|&(_, pos)| pos).collect();
    let expected: Vec<_> = (HEAD.y + 1..START.y)
        .rev()
        .map(|y| Point::new(START.x, y))
        .collect();
    assert_eq!(points, expected, "{seen:?}");
    let death: Vec<_> = seen.iter().filter(|seen| seen.sparx_death).collect();
    assert_eq!(death.len(), 1, "{seen:?}");
    let (last_tick, _) = chase[chase.len() - 1];
    assert_eq!(death[0].tick, last_tick + SUPER_SPARX_STEP_TICKS);
}

#[test]
fn a_super_sparx_steps_along_the_stix_every_super_sparx_step_ticks() {
    let mut game = game_with(SparxKind::Super);

    let chase = on_the_stix(&watch(&mut game));

    assert!(chase.len() > 1, "{chase:?}");
    for pair in chase.windows(2) {
        assert_eq!(pair[1].0 - pair[0].0, SUPER_SPARX_STEP_TICKS, "{chase:?}");
    }
}

#[test]
fn a_normal_sparx_does_not_enter_the_stix() {
    let mut game = game_with(SparxKind::Normal);

    let seen = watch(&mut game);

    assert!(seen.iter().any(|seen| seen.pos.x < START.x), "{seen:?}");
    assert_eq!(on_the_stix(&seen), [], "{seen:?}");
    assert!(seen.iter().all(|seen| !seen.sparx_death), "{seen:?}");
    assert!(
        seen.iter().all(|seen| game.board().is_border(seen.pos)),
        "{seen:?}"
    );
}

#[test]
fn a_sparx_on_a_cleared_stix_snaps_to_the_nearest_border_point() {
    let mut game = game_with(SparxKind::Super);

    let seen = watch(&mut game);

    let death = seen.iter().find(|seen| seen.sparx_death).expect("no death");
    assert!(game.stix().is_none());
    // The nearest border point to the head is beside the Qix's pen wall.
    assert_eq!(death.pos, Point::new(PEN_COLUMN + 2, HEAD.y), "{seen:?}");
}

#[test]
fn a_sparx_on_a_closed_stix_ends_on_the_border() {
    let mut game = game_with(SparxKind::Super);

    draw(&mut game, Dir::Up, DrawKind::Fast, u32::from(H - 1));
    let chaser = game.sparx()[0];
    let stix = game.stix().expect("the stix closed early");
    assert!(
        chaser.chases_along(stix) && chaser.pos != START,
        "{chaser:?}"
    );

    // The last step reaches the top edge and turns the stix into border.
    let mut events = Vec::new();
    for _ in 0..MARKER_STEP_TICKS {
        game.tick(&drawing(Dir::Up, DrawKind::Fast));
        events.extend_from_slice(game.events());
    }

    let sparx = game.sparx()[0];
    assert!(game.stix().is_none());
    assert!(game.board().is_border(sparx.pos), "{sparx:?}");
    assert_eq!(sparx.pos.x, START.x, "{sparx:?}");
    assert!(
        events.iter().all(|event| !matches!(event, Event::Death(_))),
        "{events:?}"
    );
}

#[test]
fn a_sparx_on_the_stix_snaps_to_the_border_on_a_qix_death() {
    let start = band_and_column().marker().pos;
    let chaser = Sparx {
        pos: Point::new(start.x + 1, start.y),
        dir: Dir::Left,
        kind: SparxKind::Super,
    };
    let mut game = band_and_column_with(vec![chaser]);
    draw(&mut game, Dir::Up, DrawKind::Fast, STEPS_UP);

    // Drawn fast, the marker keeps the chaser a step behind it until the
    // stix crosses the Qix's column.
    let past_column = u32::from(COLUMN - start.x) + 1;
    let before_death = (1..=past_column * MARKER_STEP_TICKS).find_map(|_| {
        let before = game.sparx()[0];
        game.tick(&drawing(Dir::Right, DrawKind::Fast));
        (!game.events().is_empty()).then_some(before)
    });

    let before = before_death.expect("the Qix never killed");
    assert_eq!(game.events(), [Event::Death(DeathCause::Qix)]);
    assert!(!game.board().is_border(before.pos), "{before:?}");
    let sparx = game.sparx()[0];
    assert!(game.board().is_border(sparx.pos), "{sparx:?}");
}

#[test]
fn a_sparx_on_the_stix_snaps_to_the_border_on_a_fuse_death() {
    let (board, qix) = penned(FUSE_W, H, PEN_COLUMN);
    let chaser = Sparx {
        pos: Point::new(FUSE_W / 2 + FUSE_GAP, H),
        dir: Dir::Left,
        kind: SparxKind::Super,
    };
    let mut game = Game::with_board(board, vec![qix], vec![chaser], Rng::new(SEED));
    draw(&mut game, Dir::Up, DrawKind::Fast, FUSE_STIX_STEPS);
    let standing = FUSE_DEATH_TICK - 1 - FUSE_STIX_STEPS * MARKER_STEP_TICKS;
    hold(&mut game, InputState::default(), standing);

    let before = game.sparx()[0];
    let stix = game.stix().expect("the marker died before the fuse");
    assert!(before.chases_along(stix), "{before:?}");
    assert!(!game.board().is_border(before.pos), "{before:?}");
    game.tick(&InputState::default());

    assert_eq!(game.events(), [Event::Death(DeathCause::Fuse)]);
    let sparx = game.sparx()[0];
    assert!(game.board().is_border(sparx.pos), "{sparx:?}");
}
