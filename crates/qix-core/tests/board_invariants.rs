//! Board invariants as `proptest` properties: random legal input
//! sequences played through [`Game::tick`] on small boards, with every
//! invariant checked after every tick.
//!
//! An input sequence is a list of [`Action`]s, each read against the game
//! it is played on: a move picks one of the marker's legal moves, a wait
//! holds nothing for a while. A case ends when the actions run out, the
//! game is over, the level is complete or a stix is caught in a dead end.

use proptest::prelude::*;
use proptest::sample::Index;
use proptest::strategy::ValueTree;
use proptest::test_runner::{FileFailurePersistence, TestCaseError, TestRunner};

use qix_core::stix::legal_moves;
use qix_core::tuning::{CLAIM_THRESHOLD_RANGE, FUSE_DELAY_TICKS, QIX_SPEED, SLOW_STEP_TICKS};
use qix_core::{
    Board, Cell, ClaimThreshold, DeathCause, Dir, DrawKind, Event, Game, InputState, MarkerState,
    Qix, Rng, Sparx, SparxKind, Stix, Vec2,
};

/// Dimensions of the boards the cases play on: small, so that random
/// stix reach the border before they wind themselves into a dead end.
const WIDTHS: std::ops::RangeInclusive<u16> = 8..=16;
const HEIGHTS: std::ops::RangeInclusive<u16> = 6..=12;

/// Actions a case plays at most.
const MAX_ACTIONS: usize = 300;

/// The odds of an action being a move against it being a wait.
const MOVE_WEIGHT: u32 = 19;
const WAIT_WEIGHT: u32 = 1;

/// Ticks a wait lasts at most: enough for the fuse to be lit and burn.
const MAX_WAIT_TICKS: u32 = 3 * FUSE_DELAY_TICKS;

/// Chance that the marker leaves the border when it can.
const START_DRAW_CHANCE: f64 = 0.3;

/// Chance that a stix keeps its heading when it can: straight runs reach
/// the border more often than a random walk.
const KEEP_HEADING_CHANCE: f64 = 0.7;

/// Length of each Qix, in playfield units: short enough for the small
/// boards.
const QIX_LENGTH: f32 = 1.0;

/// Spacing of the points the free-cell check samples along a Qix,
/// independently of the cell walk the Qix moves by.
const SAMPLE_SPACING: f32 = 0.05;

/// Cases the coverage test samples from the strategy.
const COVERAGE_CASES: u32 = 64;

/// What the cases of [`COVERAGE_CASES`] must have done at least, for the
/// invariants to have been put to the test.
const MIN_CLAIMS: u32 = 50;
const MIN_CHASING_TICKS: u32 = 1;
const MIN_LEVELS_COMPLETE: u32 = 1;
const MIN_DEATHS_BY_EACH_CAUSE: u32 = 1;

/// The game a case plays: its board, its Qix and its Sparx.
#[derive(Clone, Debug)]
struct Setup {
    width: u16,
    height: u16,
    /// The seed of the game's generator.
    seed: u64,
    /// The headings of the Qix, in radians: one or two of them, spread
    /// along the middle row.
    qix_headings: Vec<f32>,
    /// Whether the first Sparx starts Super: the second always does, so
    /// that one chases along the stix.
    first_super: bool,
}

impl Setup {
    /// A game on a fresh board, with the highest claim threshold, so the
    /// cases claim as much as they can before the level is complete.
    fn game(&self) -> Game {
        let board = Board::new(self.width, self.height);
        let qix = self.qix(&board);
        let [first, second] = Sparx::spawn_pair(&board);
        let first_kind = if self.first_super {
            SparxKind::Super
        } else {
            SparxKind::Normal
        };
        let sparx = vec![
            Sparx {
                kind: first_kind,
                ..first
            },
            Sparx {
                kind: SparxKind::Super,
                ..second
            },
        ];
        let mut game = Game::with_board(board, qix, sparx, Rng::new(self.seed));
        game.set_claim_threshold(ClaimThreshold::new(*CLAIM_THRESHOLD_RANGE.end()));
        game
    }

    /// The Qix, level and [`QIX_LENGTH`] long, spread evenly along the
    /// middle row of `board`, each moving at the slowest speed.
    fn qix(&self, board: &Board) -> Vec<Qix> {
        let (width, height) = (f32::from(board.width()), f32::from(board.height()));
        let slots = (self.qix_headings.len() + 1) as f32;
        let half = Vec2::new(QIX_LENGTH * 0.5, 0.0);
        let qix = self
            .qix_headings
            .iter()
            .enumerate()
            .map(|(slot, &heading)| {
                let centre = Vec2::new(width * (slot + 1) as f32 / slots, height * 0.5);
                let vel = Vec2::from_angle(heading) * *QIX_SPEED.start();
                Qix::new(centre - half, centre + half, vel, 0.0)
            });
        qix.collect()
    }
}

/// One part of an input sequence, read against the game it is played on.
#[derive(Clone, Copy, Debug)]
enum Action {
    /// A step of the marker, held until it moves or dies: one of its
    /// legal moves (see [`legal_move`]).
    Move(Choice),
    /// Nothing held for this many ticks.
    Wait(u32),
}

/// What picks a marker step among its legal moves.
#[derive(Clone, Copy, Debug)]
struct Choice {
    /// Picks the direction among the legal ones.
    pick: Index,
    /// Leave the border, when the marker can.
    start_draw: bool,
    /// Keep the stix's heading, when that is legal.
    keep_heading: bool,
    /// The draw button held for a stix step.
    kind: DrawKind,
}

fn setup() -> impl Strategy<Value = Setup> {
    let headings = prop::collection::vec(0.0..std::f32::consts::TAU, 1..=2);
    (WIDTHS, HEIGHTS, any::<u64>(), headings, any::<bool>()).prop_map(
        |(width, height, seed, qix_headings, first_super)| Setup {
            width,
            height,
            seed,
            qix_headings,
            first_super,
        },
    )
}

fn draw_kind() -> impl Strategy<Value = DrawKind> {
    prop_oneof![Just(DrawKind::Fast), Just(DrawKind::Slow)]
}

fn action() -> impl Strategy<Value = Action> {
    let choice = (
        any::<Index>(),
        prop::bool::weighted(START_DRAW_CHANCE),
        prop::bool::weighted(KEEP_HEADING_CHANCE),
        draw_kind(),
    )
        .prop_map(|(pick, start_draw, keep_heading, kind)| Choice {
            pick,
            start_draw,
            keep_heading,
            kind,
        });
    prop_oneof![
        MOVE_WEIGHT => choice.prop_map(Action::Move),
        WAIT_WEIGHT => (1..=MAX_WAIT_TICKS).prop_map(Action::Wait),
    ]
}

/// A game to play and the input sequence to play on it.
fn case() -> impl Strategy<Value = (Setup, Vec<Action>)> {
    (setup(), prop::collection::vec(action(), 1..=MAX_ACTIONS))
}

/// One marker step: the direction held, with the draw button, if any.
#[derive(Clone, Copy, Debug)]
struct Move {
    dir: Dir,
    draw: Option<DrawKind>,
}

impl Move {
    fn input(self) -> InputState {
        InputState {
            dir: Some(self.dir),
            fast: self.draw == Some(DrawKind::Fast),
            slow: self.draw == Some(DrawKind::Slow),
            ..InputState::default()
        }
    }
}

/// The legal move `choice` picks for `game`'s marker: a step of its stix,
/// a step off the border into free space or a step along the border; or
/// `None` when its stix is caught in a dead end.
fn legal_move(game: &Game, choice: Choice) -> Option<Move> {
    let Choice {
        pick,
        start_draw,
        keep_heading,
        kind,
    } = choice;
    let (marker, board) = (game.marker(), game.board());
    let draw = Some(kind);
    if let Some(stix) = game.stix() {
        let moves = legal_moves(&marker, stix, board);
        let heading = match stix.points() {
            [.., before, head] => Dir::between(*before, *head),
            _ => None,
        };
        let kept = heading.filter(|dir| keep_heading && moves.contains(dir));
        let dir = kept.or_else(|| pick_from(&moves, pick))?;
        return Some(Move { dir, draw });
    }
    // The marker cannot draw during the grace period after a death.
    let starts = match game.grace_ticks() {
        0 => legal_moves(&marker, &Stix::new(marker.pos), board),
        _ => Vec::new(),
    };
    if let Some(dir) = pick_from(&starts, pick).filter(|_| start_draw) {
        return Some(Move { dir, draw });
    }
    let walks: Vec<Dir> = board
        .border_neighbours(marker.pos)
        .map(|(dir, _)| dir)
        .collect();
    let dir = pick_from(&walks, pick).expect("a border point has border edges");
    Some(Move { dir, draw: None })
}

/// The item of `items` that `pick` selects, or `None` when there is none.
fn pick_from<T: Copy>(items: &[T], pick: Index) -> Option<T> {
    (!items.is_empty()).then(|| *pick.get(items))
}

/// What the ticks of one or more cases did: enough of it shows that the
/// invariants were put to the test.
#[derive(Clone, Copy, Debug, Default)]
struct Stats {
    claims: u32,
    /// Ticks after which a Sparx stood off the border, on the stix.
    chasing_ticks: u32,
    levels_complete: u32,
    /// Deaths by the Qix, the Sparx and the fuse.
    deaths: [u32; 3],
}

impl Stats {
    /// Counts what the last tick of `game` did.
    fn record(&mut self, game: &Game) {
        for event in game.events() {
            match event {
                Event::Claim { .. } => self.claims += 1,
                Event::LevelComplete { .. } => self.levels_complete += 1,
                Event::Death(cause) => self.deaths[death_index(*cause)] += 1,
                Event::Draw(_) | Event::ExtraLife | Event::GameOver => {}
            }
        }
        let board = game.board();
        let chasing = game.sparx().iter().any(|sparx| !board.is_border(sparx.pos));
        self.chasing_ticks += u32::from(chasing);
    }

    fn add(self, other: Self) -> Self {
        let mut deaths = self.deaths;
        for (total, more) in deaths.iter_mut().zip(other.deaths) {
            *total += more;
        }
        Self {
            claims: self.claims + other.claims,
            chasing_ticks: self.chasing_ticks + other.chasing_ticks,
            levels_complete: self.levels_complete + other.levels_complete,
            deaths,
        }
    }
}

/// The index of `cause` in [`Stats::deaths`].
fn death_index(cause: DeathCause) -> usize {
    match cause {
        DeathCause::Qix => 0,
        DeathCause::Sparx => 1,
        DeathCause::Fuse => 2,
    }
}

/// An invariant check, handed the number of claimed cells before the
/// tick and the game after it.
type Check = fn(usize, &Game) -> Result<(), TestCaseError>;

/// The number of claimed cells on `board`.
fn claimed_cells(board: &Board) -> usize {
    let cells = (0..board.height()).flat_map(|y| (0..board.width()).map(move |x| (x, y)));
    cells
        .filter(|&(x, y)| board.cell(i32::from(x), i32::from(y)).is_claimed())
        .count()
}

/// Ticks `game` once with `input`, then runs `check` on it.
fn tick(
    game: &mut Game,
    input: &InputState,
    check: Check,
    stats: &mut Stats,
) -> Result<(), TestCaseError> {
    let claimed = claimed_cells(game.board());
    game.tick(input);
    stats.record(game);
    check(claimed, game)
}

/// Whether the last tick of `game` killed the marker.
fn died(game: &Game) -> bool {
    game.events()
        .iter()
        .any(|event| matches!(event, Event::Death(_)))
}

/// Holds `step` until the marker moves or dies; a legal move that does
/// neither within the slowest step fails the case.
fn play_move(
    game: &mut Game,
    step: Move,
    check: Check,
    stats: &mut Stats,
) -> Result<(), TestCaseError> {
    let from = game.marker().pos;
    for _ in 0..SLOW_STEP_TICKS {
        tick(game, &step.input(), check, stats)?;
        if died(game) || game.marker().pos != from {
            return Ok(());
        }
    }
    Err(TestCaseError::fail(format!(
        "legal move {step:?} from {from:?} did not step"
    )))
}

/// Plays `actions` on the game of `setup`, running `check` after every
/// tick, until the game is over, its level complete or its stix caught
/// in a dead end.
fn play_case(setup: &Setup, actions: &[Action], check: Check) -> Result<Stats, TestCaseError> {
    let mut game = setup.game();
    let mut stats = Stats::default();
    for action in actions {
        if game.is_over() || game.is_level_complete() {
            break;
        }
        match *action {
            Action::Move(choice) => {
                let Some(step) = legal_move(&game, choice) else {
                    break;
                };
                play_move(&mut game, step, check, &mut stats)?;
            }
            Action::Wait(ticks) => {
                for _ in 0..ticks {
                    tick(&mut game, &InputState::default(), check, &mut stats)?;
                }
            }
        }
    }
    Ok(stats)
}

/// Claimed cells never become free, and the claimed fraction is the
/// claimed share of the cells.
fn check_claims(claimed_before: usize, game: &Game) -> Result<(), TestCaseError> {
    let board = game.board();
    let claimed = claimed_cells(board);
    prop_assert!(
        claimed >= claimed_before,
        "claimed cells {claimed_before} -> {claimed}"
    );
    let cells = usize::from(board.width()) * usize::from(board.height());
    // Both sides round the same quotient of two small integers, so they
    // are equal to the bit.
    prop_assert_eq!(board.claimed_fraction(), claimed as f32 / cells as f32);
    Ok(())
}

/// A stix exists exactly while the marker draws, and a marker that does
/// not draw is on the border: no stix is left after a close.
fn check_marker(game: &Game) -> Result<(), TestCaseError> {
    let marker = game.marker();
    let drawing = marker.state == MarkerState::Drawing;
    prop_assert_eq!(game.stix().is_some(), drawing, "{:?}", marker);
    if !drawing {
        prop_assert!(
            game.board().is_border(marker.pos),
            "{:?} off the border",
            marker
        );
    }
    Ok(())
}

/// Every point sampled along each Qix, and the cell under its midpoint,
/// is on a free cell.
fn check_qix(game: &Game) -> Result<(), TestCaseError> {
    let board = game.board();
    for qix in game.qix() {
        let (a, b) = (qix.a(), qix.b());
        let samples = ((b - a).length() / SAMPLE_SPACING).ceil().max(1.0) as u32;
        for i in 0..=samples {
            let point = a + (b - a) * (i as f32 / samples as f32);
            let cell = board.cell(point.x.floor() as i32, point.y.floor() as i32);
            prop_assert_eq!(cell, Cell::Free, "{:?} of {:?}", point, qix);
        }
        let (x, y) = qix.cell();
        let mid = board.cell(i32::from(x), i32::from(y));
        prop_assert_eq!(mid, Cell::Free, "midpoint of {:?}", qix);
    }
    Ok(())
}

/// Every Sparx is on the border, or chasing along the stix.
fn check_sparx(game: &Game) -> Result<(), TestCaseError> {
    for sparx in game.sparx() {
        let on_border = game.board().is_border(sparx.pos);
        let chasing = game.stix().is_some_and(|stix| sparx.chases_along(stix));
        prop_assert!(
            on_border || chasing,
            "{:?} off the border and the stix",
            sparx
        );
    }
    Ok(())
}

/// Every invariant of the game after a tick.
fn check_invariants(claimed_before: usize, game: &Game) -> Result<(), TestCaseError> {
    check_claims(claimed_before, game)?;
    check_marker(game)?;
    check_qix(game)?;
    check_sparx(game)
}

proptest! {
    // Next to this file: an integration test has no `lib.rs` for the
    // default persistence to find.
    #![proptest_config(ProptestConfig {
        failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
        ..ProptestConfig::default()
    })]

    #[test]
    fn every_tick_keeps_the_invariants((setup, actions) in case()) {
        play_case(&setup, &actions, check_invariants)?;
    }
}

#[test]
fn the_cases_exercise_what_the_invariants_check() {
    let mut runner = TestRunner::deterministic();
    let strategy = case();
    let mut total = Stats::default();
    for _ in 0..COVERAGE_CASES {
        let (setup, actions) = strategy.new_tree(&mut runner).expect("a case").current();
        let stats = play_case(&setup, &actions, |_, _| Ok(())).expect("no check to fail");
        total = total.add(stats);
    }
    assert!(total.claims >= MIN_CLAIMS, "{total:?}");
    assert!(total.chasing_ticks >= MIN_CHASING_TICKS, "{total:?}");
    assert!(total.levels_complete >= MIN_LEVELS_COMPLETE, "{total:?}");
    let each_cause = total
        .deaths
        .iter()
        .all(|&deaths| deaths >= MIN_DEATHS_BY_EACH_CAUSE);
    assert!(each_cause, "{total:?}");
}
