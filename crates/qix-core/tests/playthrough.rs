//! Scripted deterministic playthroughs: fixed sequences of [`InputState`]s
//! fed to a [`Session`] over a full-size [`Game`] with a fixed seed, from
//! the title screen through a completed level and a game over to the high
//! score table.
//!
//! The scripts are tuned for [`SEED`]: a change to the rules, the tuning
//! or the order the game draws from its [`Rng`] changes where the Qix
//! and the Sparx go, so these tests are expected to need a new seed or
//! script when that happens on purpose.

use qix_core::tuning::{
    BOARD_H, BOARD_W, BONUS_PER_PERCENT, CLAIM_THRESHOLD_DEFAULT, EXTRA_LIFE_EVERY, FIRST_LEVEL,
    FUSE_DELAY_TICKS, FUSE_STEP_TICKS, MARKER_STEP_TICKS, POINTS_PER_CELL, RESPAWN_GRACE_TICKS,
    STARTING_LIVES,
};
use qix_core::{
    DeathCause, DemoStep, Dir, DrawKind, Event, Game, GameState, HighScore, Initials, InputState,
    Rng, Session,
};

/// Seed of every game the playthroughs play.
const SEED: u64 = 0;

/// The column of the marker's start, the centre of the bottom edge.
const START_X: u32 = BOARD_W as u32 / 2;

/// The column of the first stix, up the whole board: the Qix is right of
/// it, so the cells left of it are claimed.
const LEFT_LINE_X: u32 = 56;

/// The column of the second stix, down the whole board: the Qix is left
/// of it, so the cells right of it are claimed.
const RIGHT_LINE_X: u32 = 104;

/// The row of the third stix, across the free columns between the first
/// two: the Qix is below it, so the cells above it are claimed.
const CUT_Y: u32 = 60;

/// Lattice steps of the stix up the middle that the Qix runs into on the
/// second level: up to the middle row, where it starts.
const INTO_QIX_STEPS: u32 = BOARD_H as u32 / 2;

/// Lattice steps of the short stix the marker stops on, which the fuse
/// burns along.
const FUSE_STIX_STEPS: u32 = 4;

/// Ticks the marker stands still on its stix: exactly as many as the
/// fuse needs to be lit and to burn to the marker. The fuse burns one
/// stix segment every [`FUSE_STEP_TICKS`], and the straight stix of
/// [`FUSE_STIX_STEPS`] lattice steps is a single segment.
const STAND_TICKS: u32 = FUSE_DELAY_TICKS + FUSE_STEP_TICKS;

/// Ticks of the grace period after a death that the script sits out.
const GRACE_WAIT_TICKS: u32 = RESPAWN_GRACE_TICKS;

/// Ticks the marker waits on the border for the Sparx to take every
/// life left: far more than they need.
const WAIT_FOR_SPARX_TICKS: u32 = 20_000;

/// The cells each stix of the first level claims, in order.
const LEVEL_ONE_CLAIMS: [u32; 3] = [
    LEFT_LINE_X * BOARD_H as u32,
    (BOARD_W as u32 - RIGHT_LINE_X) * BOARD_H as u32,
    (RIGHT_LINE_X - LEFT_LINE_X) * CUT_Y,
];

/// The first level's claimed share at its end, in whole percent points.
const LEVEL_ONE_PERCENT: u32 = 85;

/// Percent points in the whole board.
const PERCENT: u32 = 100;

/// The deaths of the second level, in order: the fuse of the stix the
/// marker stops on, the Qix the next stix runs into, then the Sparx for
/// every life left.
const LEVEL_TWO_DEATHS: [DeathCause; 8] = [
    DeathCause::Fuse,
    DeathCause::Qix,
    DeathCause::Sparx,
    DeathCause::Sparx,
    DeathCause::Sparx,
    DeathCause::Sparx,
    DeathCause::Sparx,
    DeathCause::Sparx,
];

/// A press of confirm, for one tick.
const CONFIRM: DemoStep = DemoStep {
    input: InputState {
        dir: None,
        fast: false,
        slow: false,
        confirm: true,
        pause: false,
    },
    ticks: 1,
};

/// A walk of `steps` lattice steps along the border.
const fn walk(dir: Dir, steps: u32) -> DemoStep {
    DemoStep::walk(dir, steps * MARKER_STEP_TICKS)
}

/// A fast draw of `steps` lattice steps.
const fn fast(dir: Dir, steps: u32) -> DemoStep {
    DemoStep::draw(dir, DrawKind::Fast, steps * MARKER_STEP_TICKS)
}

/// From the title screen, the first level: a stix up column
/// [`LEFT_LINE_X`], one down column [`RIGHT_LINE_X`], then one along row
/// [`CUT_Y`] between them, from the second back to the first.
const LEVEL_ONE: &[DemoStep] = &[
    CONFIRM,
    walk(Dir::Left, START_X - LEFT_LINE_X),
    fast(Dir::Up, BOARD_H as u32),
    walk(Dir::Right, RIGHT_LINE_X - LEFT_LINE_X),
    fast(Dir::Down, BOARD_H as u32),
    walk(Dir::Up, BOARD_H as u32 - CUT_Y),
    fast(Dir::Left, RIGHT_LINE_X - LEFT_LINE_X),
];

/// From the end of the first level, the start of the second until the
/// fuse kills: a short stix the marker stops on.
const LEVEL_TWO_FUSE: &[DemoStep] = &[
    CONFIRM,
    fast(Dir::Up, FUSE_STIX_STEPS),
    DemoStep::idle(STAND_TICKS),
];

/// The rest of the second level, until the game is over: the grace
/// period, a stix into the Qix, then a wait on the border.
const LEVEL_TWO_REST: &[DemoStep] = &[
    DemoStep::idle(GRACE_WAIT_TICKS),
    fast(Dir::Up, INTO_QIX_STEPS),
    DemoStep::idle(WAIT_FOR_SPARX_TICKS),
];

/// From the game over screen, the initials of a first entry in the table:
/// confirm moves on to the initials, then confirms each of their letters
/// as it starts.
const ENTER_INITIALS: &[DemoStep] = &[CONFIRM, CONFIRM, CONFIRM, CONFIRM];

/// A session on the title screen, over a new full-size game seeded with
/// [`SEED`].
fn new_session() -> Session {
    Session::new(Game::new(Rng::new(SEED)))
}

/// Plays `script` on `session` and returns the events of every tick, in
/// order, but for the stix steps.
fn play(session: &mut Session, script: &[DemoStep]) -> Vec<Event> {
    let mut events = Vec::new();
    for step in script {
        for _ in 0..step.ticks {
            session.tick(&step.input);
            let raised = session.events().iter();
            events.extend(raised.filter(|event| !matches!(event, Event::Draw(_))));
        }
    }
    events
}

/// The whole second level: [`LEVEL_TWO_FUSE`], then [`LEVEL_TWO_REST`].
fn level_two() -> Vec<DemoStep> {
    [LEVEL_TWO_FUSE, LEVEL_TWO_REST].concat()
}

/// The cells of the claims among `events`.
fn claims(events: &[Event]) -> Vec<u32> {
    let cells = events.iter().filter_map(|event| match *event {
        Event::Claim { cells, .. } => u32::try_from(cells).ok(),
        _ => None,
    });
    cells.collect()
}

/// The causes of the deaths among `events`.
fn deaths(events: &[Event]) -> Vec<DeathCause> {
    let causes = events.iter().filter_map(|event| match *event {
        Event::Death(cause) => Some(cause),
        _ => None,
    });
    causes.collect()
}

/// The number of [`Event::ExtraLife`] among `events`.
fn extra_lives(events: &[Event]) -> u32 {
    let earned = events.iter().filter(|event| **event == Event::ExtraLife);
    u32::try_from(earned.count()).unwrap_or(u32::MAX)
}

/// The bonus of the first level: every whole percent point above the
/// default threshold.
fn level_one_bonus() -> u32 {
    let threshold = (CLAIM_THRESHOLD_DEFAULT * PERCENT as f32).round() as u32;
    (LEVEL_ONE_PERCENT - threshold) * BONUS_PER_PERCENT
}

/// The score at the end of the first level: every claimed cell at the
/// starting multiplier of one, plus the level bonus.
fn level_one_score() -> u32 {
    let cells: u32 = LEVEL_ONE_CLAIMS.iter().sum();
    cells * POINTS_PER_CELL + level_one_bonus()
}

#[test]
fn the_first_level_is_completed_without_a_death() {
    let mut session = new_session();
    let events = play(&mut session, LEVEL_ONE);
    let score = level_one_score();
    let earned = score / EXTRA_LIFE_EVERY;
    assert_eq!(claims(&events), LEVEL_ONE_CLAIMS);
    assert_eq!(deaths(&events), []);
    assert_eq!(extra_lives(&events), earned);
    assert_eq!(
        session.state(),
        GameState::LevelComplete {
            fraction: LEVEL_ONE_PERCENT as f32 / PERCENT as f32,
            bonus: level_one_bonus(),
            split: false,
        }
    );
    let game = session.game();
    assert_eq!(game.level(), FIRST_LEVEL);
    assert_eq!(game.score().points(), score);
    assert_eq!(game.lives(), STARTING_LIVES + earned);
}

#[test]
fn the_second_level_loses_every_life_and_scores_nothing() {
    let mut session = new_session();
    play(&mut session, LEVEL_ONE);
    let lives = session.game().lives();
    let events = play(&mut session, &level_two());
    assert_eq!(deaths(&events), LEVEL_TWO_DEATHS);
    assert_eq!(LEVEL_TWO_DEATHS.len(), lives as usize);
    assert_eq!(claims(&events), []);
    assert_eq!(events.last(), Some(&Event::GameOver));
    assert_eq!(session.state(), GameState::GameOver);
    let game = session.game();
    assert_eq!(game.level(), FIRST_LEVEL + 1);
    assert_eq!(game.score().points(), level_one_score());
    assert_eq!(game.lives(), 0);
}

#[test]
fn the_fuse_reaches_the_marker_on_the_last_stand_tick() {
    let mut session = new_session();
    play(&mut session, LEVEL_ONE);
    let (last_tick, before) = LEVEL_TWO_FUSE.split_last().expect("a stand");
    let early = DemoStep::idle(last_tick.ticks - 1);
    let events = play(&mut session, &[before, &[early]].concat());
    assert_eq!(deaths(&events), []);
    let events = play(&mut session, &[DemoStep::idle(1)]);
    assert_eq!(events, [Event::Death(DeathCause::Fuse)]);
}

#[test]
fn the_final_score_enters_the_high_score_table() {
    let mut session = new_session();
    play(&mut session, LEVEL_ONE);
    play(&mut session, &level_two());
    play(&mut session, ENTER_INITIALS);
    assert_eq!(session.state(), GameState::Title);
    let entry = HighScore {
        name: Initials::FIRST,
        score: level_one_score(),
    };
    assert_eq!(session.high_scores().entries(), [entry]);
    assert_eq!(session.high_score(), level_one_score());
    let game = session.game();
    assert_eq!(game.level(), FIRST_LEVEL);
    assert_eq!(game.score().points(), 0);
    assert_eq!(game.lives(), STARTING_LIVES);
}

#[test]
fn the_same_script_plays_the_same_game_twice() {
    let whole = [LEVEL_ONE, &level_two()].concat();
    let mut first = new_session();
    let mut second = new_session();
    assert_eq!(play(&mut first, &whole), play(&mut second, &whole));
    assert_eq!(first.game().ticks(), second.game().ticks());
    assert_eq!(first.game().board(), second.game().board());
    assert_eq!(first.game().marker(), second.game().marker());
}
