//! The eframe application: samples input, runs the core ticks each frame
//! owes, and paints the playfield with the screen of the session's state.

use std::time::{SystemTime, UNIX_EPOCH};

use eframe::egui;
use qix_core::{Game, Rng, Session};

use crate::input::{self, InputLatch};
use crate::playfield;
use crate::render::screen;
use crate::timestep::FixedTimestep;

/// The application state kept between frames.
#[derive(Debug)]
pub struct QixApp {
    session: Session,
    timestep: FixedTimestep,
    input: InputLatch,
    /// Whether the debug overlay is shown.
    overlay: bool,
}

impl Default for QixApp {
    /// The title screen of a new game seeded from the clock, so each run
    /// plays differently.
    fn default() -> Self {
        Self {
            session: Session::new(Game::new(Rng::new(clock_seed()))),
            timestep: FixedTimestep::default(),
            input: InputLatch::default(),
            overlay: false,
        }
    }
}

/// The nanoseconds since the Unix epoch, truncated to their low 64 bits;
/// zero on a clock set before it.
fn clock_seed() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_nanos() as u64)
}

impl QixApp {
    /// Samples the keyboard and runs as many ticks as the frame's time owes.
    fn advance(&mut self, ctx: &egui::Context) {
        let (sample, overlay_toggled, dt) =
            ctx.input(|i| (input::sample(i), input::overlay_toggled(i), i.stable_dt));
        self.overlay ^= overlay_toggled;
        self.input.record(sample);
        for _ in 0..self.timestep.advance(dt) {
            let tick_input = self.input.next_tick();
            self.session.tick(&tick_input);
        }
    }
}

impl eframe::App for QixApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.advance(ui.ctx());
        egui::CentralPanel::default()
            .frame(playfield::panel_frame())
            .show(ui, |ui| {
                let screen = screen::screen(&self.session);
                playfield::show(ui, self.session.game(), screen.as_ref(), self.overlay);
            });
        // The game moves on its own, so every frame is followed by another.
        ui.ctx().request_repaint();
    }
}
