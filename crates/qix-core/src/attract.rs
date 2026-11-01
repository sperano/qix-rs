//! Attract mode: the title screen starts a demo after sitting idle.

use crate::demo::Demo;
use crate::input::InputState;
use crate::tuning::ATTRACT_DELAY_TICKS;

/// Counts the idle ticks of the title screen and runs the [`Demo`] once
/// there are [`ATTRACT_DELAY_TICKS`] of them.
#[derive(Debug, Default)]
pub struct Attract {
    /// Ticks since the last input, or the end of the last demo.
    idle_ticks: u32,
    /// The demo being shown, if any.
    demo: Option<Demo>,
}

impl Attract {
    /// Attract mode with the title screen just up: no demo yet.
    pub fn new() -> Self {
        Self::default()
    }

    /// Advances attract mode one tick of the title screen with `input`.
    ///
    /// Any input stops a running demo and restarts the idle count.
    /// Otherwise a running demo ticks, and is dropped once finished, which
    /// restarts the idle count; with no demo, the tick counts as idle and
    /// the [`ATTRACT_DELAY_TICKS`]th starts one.
    pub fn tick(&mut self, input: &InputState) {
        if *input != InputState::default() {
            self.reset();
        } else if let Some(demo) = &mut self.demo {
            demo.tick();
            if demo.is_finished() {
                self.reset();
            }
        } else {
            self.idle_ticks += 1;
            if self.idle_ticks >= ATTRACT_DELAY_TICKS {
                self.demo = Some(Demo::new());
            }
        }
    }

    /// The demo being shown, if any.
    pub fn demo(&self) -> Option<&Demo> {
        self.demo.as_ref()
    }

    /// Drops the demo and restarts the idle count: for when the title
    /// screen is left or the settings open.
    pub fn reset(&mut self) {
        self.idle_ticks = 0;
        self.demo = None;
    }
}
