//! The eframe application: samples input, runs the core ticks each frame
//! owes, paints the playfield with the HUD and the screen of the
//! session's state, and keeps the high score table in eframe's storage.

use std::time::{SystemTime, UNIX_EPOCH};

use eframe::{Storage, egui};
use qix_core::{Game, HighScores, Rng, Session};

use crate::input::{self, InputLatch};
use crate::persist;
use crate::playfield;
use crate::render::{hud, screen};
use crate::timestep::FixedTimestep;

/// The application state kept between frames.
#[derive(Debug)]
pub struct QixApp {
    session: Session,
    timestep: FixedTimestep,
    input: InputLatch,
    /// Whether the debug overlay is shown.
    overlay: bool,
    /// The high score table as last written to the storage.
    saved_high_scores: HighScores,
}

impl QixApp {
    /// The title screen of a new game seeded from the clock, so each run
    /// plays differently, with the high score table kept in `storage`.
    pub fn new(storage: Option<&dyn Storage>) -> Self {
        let high_scores = persist::load_high_scores(storage);
        Self {
            session: Session::with_high_scores(
                Game::new(Rng::new(clock_seed())),
                high_scores.clone(),
            ),
            timestep: FixedTimestep::default(),
            input: InputLatch::default(),
            overlay: false,
            saved_high_scores: high_scores,
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

    /// Writes the high score table to `frame`'s storage and flushes it as
    /// soon as an entry changes it, rather than at eframe's next save.
    fn save_new_high_scores(&mut self, frame: &mut eframe::Frame) {
        save_if_changed(
            frame
                .storage_mut()
                .map(|storage| storage as &mut dyn Storage),
            &mut self.saved_high_scores,
            self.session.high_scores(),
        );
    }
}

/// Writes `high_scores` to `storage` and flushes it when they differ from
/// `saved`, the table last written, then records them as saved; does
/// nothing when they are the same.
pub fn save_if_changed(
    storage: Option<&mut dyn Storage>,
    saved: &mut HighScores,
    high_scores: &HighScores,
) {
    if high_scores == saved {
        return;
    }
    if let Some(storage) = storage {
        persist::save_high_scores(storage, high_scores);
        storage.flush();
    }
    saved.clone_from(high_scores);
}

impl eframe::App for QixApp {
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        self.advance(ui.ctx());
        self.save_new_high_scores(frame);
        egui::CentralPanel::default()
            .frame(playfield::panel_frame())
            .show(ui, |ui| {
                let game = self.session.game();
                let hud = hud::hud(game, self.session.high_score());
                let screen = screen::screen(&self.session);
                playfield::show(ui, game, &hud, screen.as_ref(), self.overlay);
            });
        // The game moves on its own, so every frame is followed by another.
        ui.ctx().request_repaint();
    }

    fn save(&mut self, storage: &mut dyn Storage) {
        persist::save_high_scores(storage, self.session.high_scores());
    }
}
