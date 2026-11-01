//! The attract-mode demo: a scripted sequence of inputs replayed through a
//! real [`Game`].

use crate::board::DrawKind;
use crate::game::Game;
use crate::geometry::Dir;
use crate::input::InputState;
use crate::rng::Rng;
use crate::tuning::{DEMO_SEED, MARKER_STEP_TICKS, SLOW_STEP_TICKS, TICK_HZ};

/// Height of the rectangles the demo claims, in cells.
pub(crate) const BOX_HEIGHT: u32 = 45;

/// Width of the first two rectangles the demo claims, in cells.
pub(crate) const BOX_WIDTH: u32 = 25;

/// Width of the last rectangle, from the border to the left edge, in
/// cells.
pub(crate) const LAST_BOX_WIDTH: u32 = 20;

/// Lattice steps the marker walks along the border between two
/// rectangles.
const HOP: u32 = 5;

/// Ticks the demo waits before its first move: two seconds.
const OPENING_TICKS: u32 = 2 * TICK_HZ;

/// Ticks the marker rests after each claim: half a second.
const REST_TICKS: u32 = TICK_HZ / 2;

/// Ticks the demo holds its last claim before it ends: a second and a
/// half.
const CLOSING_TICKS: u32 = 3 * TICK_HZ / 2;

/// One part of a demo script: an input held for a number of ticks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DemoStep {
    /// The input every tick of the step sees.
    pub input: InputState,
    /// How many ticks the input is held.
    pub ticks: u32,
}

impl DemoStep {
    /// No control held for `ticks` ticks.
    pub const fn idle(ticks: u32) -> Self {
        Self {
            input: InputState {
                dir: None,
                fast: false,
                slow: false,
                confirm: false,
                pause: false,
            },
            ticks,
        }
    }

    /// `dir` held, without a draw button, for `ticks` ticks.
    pub const fn walk(dir: Dir, ticks: u32) -> Self {
        let mut step = Self::idle(ticks);
        step.input.dir = Some(dir);
        step
    }

    /// `dir` held with the draw button of `kind` for `ticks` ticks.
    pub const fn draw(dir: Dir, kind: DrawKind, ticks: u32) -> Self {
        let mut step = Self::walk(dir, ticks);
        match kind {
            DrawKind::Fast => step.input.fast = true,
            DrawKind::Slow => step.input.slow = true,
        }
        step
    }
}

/// A walk of `steps` lattice steps along the border.
const fn walk(dir: Dir, steps: u32) -> DemoStep {
    DemoStep::walk(dir, steps * MARKER_STEP_TICKS)
}

/// A fast draw of `steps` lattice steps.
const fn fast(dir: Dir, steps: u32) -> DemoStep {
    DemoStep::draw(dir, DrawKind::Fast, steps * MARKER_STEP_TICKS)
}

/// A slow draw of `steps` lattice steps.
const fn slow(dir: Dir, steps: u32) -> DemoStep {
    DemoStep::draw(dir, DrawKind::Slow, steps * SLOW_STEP_TICKS)
}

/// The demo's inputs, for a [`Game`] seeded with [`DEMO_SEED`]. The
/// marker starts at the centre of the bottom edge and claims three
/// rectangles along it, working leftwards: a fast one [`BOX_WIDTH`]
/// cells wide and [`BOX_HEIGHT`] high, a slow one of the same size and a
/// slow one [`LAST_BOX_WIDTH`] wide against the left edge. A sixth of the
/// board is claimed at the end.
pub const DEMO_SCRIPT: &[DemoStep] = &[
    DemoStep::idle(OPENING_TICKS),
    fast(Dir::Up, BOX_HEIGHT),
    fast(Dir::Left, BOX_WIDTH),
    fast(Dir::Down, BOX_HEIGHT),
    DemoStep::idle(REST_TICKS),
    walk(Dir::Left, HOP),
    slow(Dir::Up, BOX_HEIGHT),
    slow(Dir::Left, BOX_WIDTH),
    slow(Dir::Down, BOX_HEIGHT),
    DemoStep::idle(REST_TICKS),
    walk(Dir::Left, HOP),
    slow(Dir::Up, BOX_HEIGHT),
    slow(Dir::Left, LAST_BOX_WIDTH),
    DemoStep::idle(CLOSING_TICKS),
];

/// A game replaying a script of inputs, one step of the script per tick.
#[derive(Debug)]
pub struct Demo {
    game: Game,
    script: &'static [DemoStep],
    /// The index of the step being replayed.
    step: usize,
    /// Ticks of that step already run.
    step_ticks: u32,
}

impl Demo {
    /// The demo of [`DEMO_SCRIPT`] on a new game seeded with
    /// [`DEMO_SEED`].
    pub fn new() -> Self {
        Self::with_script(Game::new(Rng::new(DEMO_SEED)), DEMO_SCRIPT)
    }

    /// A demo replaying `script` on `game`.
    pub fn with_script(game: Game, script: &'static [DemoStep]) -> Self {
        let mut demo = Self {
            game,
            script,
            step: 0,
            step_ticks: 0,
        };
        demo.skip_empty_steps();
        demo
    }

    /// The game being played.
    pub fn game(&self) -> &Game {
        &self.game
    }

    /// Ticks the game once with the input of the current step and moves
    /// on in the script. Does nothing once the demo is finished.
    pub fn tick(&mut self) {
        if self.is_finished() {
            return;
        }
        let step = self.script[self.step];
        self.game.tick(&step.input);
        self.step_ticks += 1;
        if self.step_ticks >= step.ticks {
            self.step += 1;
            self.step_ticks = 0;
            self.skip_empty_steps();
        }
    }

    /// The script has run out, or the game is over or its level
    /// complete.
    pub fn is_finished(&self) -> bool {
        self.step >= self.script.len() || self.game.is_over() || self.game.is_level_complete()
    }

    /// Moves past steps that hold their input for no tick.
    fn skip_empty_steps(&mut self) {
        while self
            .script
            .get(self.step)
            .is_some_and(|step| step.ticks == 0)
        {
            self.step += 1;
        }
    }
}

impl Default for Demo {
    fn default() -> Self {
        Self::new()
    }
}
