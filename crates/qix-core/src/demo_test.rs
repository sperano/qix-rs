use crate::qix_test::qix_at;
use crate::tuning::{BOARD_H, BOARD_W, MARKER_STEP_TICKS, SLOW_STEP_TICKS};
use crate::{
    Board, Cell, ClaimThreshold, DEMO_SCRIPT, Demo, DemoStep, Dir, DrawKind, Event, Game,
    InputState, Point, Rng, Sparx, SparxKind,
};

/// Seed of the games the custom-script tests play.
const SEED: u64 = 1194;

/// Ticks of the idle step of the short scripts.
const IDLE_TICKS: u32 = 5;

/// Steps of the stix up the middle column of [`short_game`].
const SHORT_STIX_STEPS: u32 = 3;

/// Dimensions of the small board of the short-script tests.
const SHORT_W: u16 = 8;
const SHORT_H: u16 = 3;

/// A script that completes the level of [`short_game`], then idles.
const SHORT_SCRIPT: &[DemoStep] = &[
    DemoStep::draw(
        Dir::Up,
        DrawKind::Fast,
        SHORT_STIX_STEPS * MARKER_STEP_TICKS,
    ),
    DemoStep::idle(IDLE_TICKS),
];

/// A script of one idle step.
const IDLE_SCRIPT: &[DemoStep] = &[DemoStep::idle(IDLE_TICKS)];

/// A script of an empty step between two idle ones.
const EMPTY_STEP_SCRIPT: &[DemoStep] = &[
    DemoStep::idle(0),
    DemoStep::idle(IDLE_TICKS),
    DemoStep::idle(0),
];

/// A script of idle ticks, longer than the game the Sparx of
/// [`sparx_game`] needs to take every life.
const LONG_IDLE_SCRIPT: &[DemoStep] = &[DemoStep::idle(100_000)];

/// A small game with a short Qix in its top-left cell: the stix up the
/// middle column claims the right half, which completes the level at the
/// lowest claim threshold.
fn short_game() -> Game {
    let mut game = Game::with_board(
        Board::new(SHORT_W, SHORT_H),
        vec![qix_at((0, 0))],
        Vec::new(),
        Rng::new(SEED),
    );
    game.set_claim_threshold(ClaimThreshold::new(0.0));
    game
}

/// A game whose Sparx stands next to the marker and kills it again and
/// again.
fn sparx_game() -> Game {
    let sparx = Sparx {
        pos: Point::new(SHORT_W / 2 + 1, SHORT_H),
        dir: Dir::Left,
        kind: SparxKind::Normal,
    };
    Game::with_board(
        Board::new(SHORT_W, SHORT_H),
        vec![qix_at((0, 0))],
        vec![sparx],
        Rng::new(SEED),
    )
}

/// Ticks `demo` to its end and returns the deaths it saw.
fn run(demo: &mut Demo) -> usize {
    let mut deaths = 0;
    while !demo.is_finished() {
        demo.tick();
        deaths += demo
            .game()
            .events()
            .iter()
            .filter(|event| matches!(event, Event::Death(_)))
            .count();
    }
    deaths
}

/// The cells of `game`'s board claimed with `kind`.
fn claimed(game: &Game, kind: DrawKind) -> usize {
    let board = game.board();
    (0..i32::from(BOARD_H))
        .flat_map(|y| (0..i32::from(BOARD_W)).map(move |x| (x, y)))
        .filter(|&(x, y)| board.cell(x, y) == Cell::Claimed(kind))
        .count()
}

fn script_ticks(script: &[DemoStep]) -> u64 {
    script.iter().map(|step| u64::from(step.ticks)).sum()
}

#[test]
fn idle_step_holds_nothing() {
    let step = DemoStep::idle(IDLE_TICKS);

    assert_eq!(step.input, InputState::default());
    assert_eq!(step.ticks, IDLE_TICKS);
}

#[test]
fn walk_step_holds_only_the_direction() {
    let step = DemoStep::walk(Dir::Left, IDLE_TICKS);

    let expected = InputState {
        dir: Some(Dir::Left),
        ..InputState::default()
    };
    assert_eq!(step.input, expected);
    assert_eq!(step.ticks, IDLE_TICKS);
}

#[test]
fn draw_step_holds_the_button_of_its_kind() {
    let fast = DemoStep::draw(Dir::Up, DrawKind::Fast, IDLE_TICKS).input;
    let slow = DemoStep::draw(Dir::Up, DrawKind::Slow, IDLE_TICKS).input;

    assert_eq!(
        (fast.dir, fast.fast, fast.slow),
        (Some(Dir::Up), true, false)
    );
    assert_eq!(
        (slow.dir, slow.fast, slow.slow),
        (Some(Dir::Up), false, true)
    );
}

#[test]
fn the_script_runs_without_losing_a_life() {
    let mut demo = Demo::new();

    let deaths = run(&mut demo);

    assert_eq!(deaths, 0);
    assert!(!demo.game().is_over());
}

#[test]
fn the_demo_finishes_exactly_after_the_ticks_of_its_script() {
    let mut demo = Demo::new();
    let total = script_ticks(DEMO_SCRIPT);

    for _ in 1..total {
        demo.tick();
        assert!(
            !demo.is_finished(),
            "finished at tick {}",
            demo.game().ticks()
        );
    }
    demo.tick();

    assert!(demo.is_finished());
    assert_eq!(demo.game().ticks(), total);
}

#[test]
fn the_script_claims_fast_and_slow_cells_below_the_threshold() {
    let mut demo = Demo::new();

    run(&mut demo);

    let game = demo.game();
    assert!(claimed(game, DrawKind::Fast) > 0);
    assert!(claimed(game, DrawKind::Slow) > 0);
    assert!(!game.is_level_complete());
    let fraction = game.board().claimed_fraction();
    assert!(
        !game.claim_threshold().is_reached(fraction),
        "claimed {fraction}"
    );
}

#[test]
fn the_script_ends_with_no_stix_half_drawn() {
    let mut demo = Demo::new();

    run(&mut demo);

    assert_eq!(demo.game().stix(), None);
}

#[test]
fn the_script_lasts_twenty_to_thirty_seconds() {
    let seconds = script_ticks(DEMO_SCRIPT) as f32 / crate::tuning::TICK_HZ as f32;

    assert!((20.0..=30.0).contains(&seconds), "{seconds} s");
}

#[test]
fn the_script_draws_slow_stix_at_the_slow_step_rate() {
    let slow_ticks: u32 = DEMO_SCRIPT
        .iter()
        .filter(|step| step.input.slow)
        .map(|step| step.ticks)
        .sum();

    assert!(slow_ticks > 0);
    assert_eq!(slow_ticks % SLOW_STEP_TICKS, 0);
}

#[test]
fn two_demos_play_the_same_game() {
    let (mut first, mut second) = (Demo::new(), Demo::new());

    run(&mut first);
    run(&mut second);

    assert_eq!(first.game().board(), second.game().board());
    assert_eq!(first.game().score(), second.game().score());
}

#[test]
fn a_finished_demo_does_not_tick() {
    let mut demo = Demo::with_script(short_game(), SHORT_SCRIPT);
    run(&mut demo);
    let ticks = demo.game().ticks();

    demo.tick();

    assert!(demo.is_finished());
    assert_eq!(demo.game().ticks(), ticks);
}

#[test]
fn a_demo_ticks_one_step_of_its_script_per_tick() {
    let mut demo = Demo::with_script(short_game(), SHORT_SCRIPT);

    demo.tick();

    assert_eq!(demo.game().ticks(), 1);
    assert!(!demo.is_finished());
}

#[test]
fn a_script_that_runs_out_finishes_the_demo() {
    let mut demo = Demo::with_script(sparx_free_game(), IDLE_SCRIPT);

    run(&mut demo);

    assert_eq!(demo.game().ticks(), u64::from(IDLE_TICKS));
    assert!(!demo.game().is_over());
    assert!(!demo.game().is_level_complete());
}

/// A game no Sparx can kill in.
fn sparx_free_game() -> Game {
    Game::with_board(
        Board::new(SHORT_W, SHORT_H),
        vec![qix_at((0, 0))],
        Vec::new(),
        Rng::new(SEED),
    )
}

#[test]
fn an_empty_script_is_finished() {
    let demo = Demo::with_script(sparx_free_game(), &[]);

    assert!(demo.is_finished());
}

#[test]
fn steps_of_no_ticks_are_skipped() {
    let mut demo = Demo::with_script(sparx_free_game(), EMPTY_STEP_SCRIPT);

    run(&mut demo);

    assert_eq!(demo.game().ticks(), u64::from(IDLE_TICKS));
}

#[test]
fn a_demo_whose_game_is_over_is_finished() {
    let mut demo = Demo::with_script(sparx_game(), LONG_IDLE_SCRIPT);

    let deaths = run(&mut demo);

    assert!(demo.game().is_over());
    assert!(demo.is_finished());
    assert_eq!(deaths, crate::tuning::STARTING_LIVES as usize);
    assert!(demo.game().ticks() < script_ticks(LONG_IDLE_SCRIPT));
}

#[test]
fn a_demo_whose_level_is_complete_is_finished_before_its_script_ends() {
    let mut demo = Demo::with_script(short_game(), SHORT_SCRIPT);

    run(&mut demo);

    assert!(demo.game().is_level_complete());
    assert!(demo.is_finished());
    assert_eq!(
        demo.game().ticks(),
        u64::from(SHORT_STIX_STEPS * MARKER_STEP_TICKS)
    );
}

#[test]
fn the_default_demo_is_the_new_one() {
    let mut demo = Demo::default();

    run(&mut demo);

    let mut new = Demo::new();
    run(&mut new);
    assert_eq!(demo.game().board(), new.game().board());
}
