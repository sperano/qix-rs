//! The eframe application: samples input, runs the core ticks each frame
//! owes, and paints the playfield.

use eframe::egui;
use qix_core::Game;

use crate::input::{self, InputLatch};
use crate::playfield;
use crate::timestep::FixedTimestep;

/// The application state kept between frames.
#[derive(Debug, Default)]
pub struct QixApp {
    game: Game,
    timestep: FixedTimestep,
    input: InputLatch,
}

impl QixApp {
    /// Samples the keyboard and runs as many ticks as the frame's time owes.
    fn advance(&mut self, ctx: &egui::Context) {
        let (sample, dt) = ctx.input(|i| (input::sample(i), i.stable_dt));
        self.input.record(sample);
        for _ in 0..self.timestep.advance(dt) {
            let tick_input = self.input.next_tick();
            self.game.tick(&tick_input);
        }
    }
}

impl eframe::App for QixApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.advance(ui.ctx());
        egui::CentralPanel::default()
            .frame(playfield::panel_frame())
            .show(ui, playfield::show);
        // The game moves on its own, so every frame is followed by another.
        ui.ctx().request_repaint();
    }
}
