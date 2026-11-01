//! The eframe application: samples input, runs the core ticks each frame
//! owes, paints the playfield with the HUD and the screen of the
//! session's state or the settings, and keeps the high score table and
//! the settings in eframe's storage.

use std::time::{SystemTime, UNIX_EPOCH};

use eframe::{Storage, egui};
use qix_core::{Game, GameState, HighScores, Rng, Session};

use crate::input::{self, InputLatch};
use crate::persist;
use crate::playfield;
use crate::render::{hud, screen, settings::settings_screen};
use crate::settings::Settings;
use crate::settings_menu::SettingsMenu;
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
    /// The settings in force, as last written to the storage.
    settings: Settings,
    /// The settings screen, while it is open over the title screen.
    menu: Option<SettingsMenu>,
    /// The window is to take the size of the settings' scale on the next
    /// frame.
    resize_window: bool,
}

impl QixApp {
    /// The title screen of a new game seeded from the clock, so each run
    /// plays differently, with the high score table and the settings kept
    /// in `storage`. The window takes the settings' size on the first
    /// frame.
    pub fn new(storage: Option<&dyn Storage>) -> Self {
        let high_scores = persist::load_high_scores(storage);
        let settings = persist::load_settings(storage);
        let mut session =
            Session::with_high_scores(Game::new(Rng::new(clock_seed())), high_scores.clone());
        session.set_rules(settings.rules());
        Self {
            session,
            timestep: FixedTimestep::default(),
            input: InputLatch::default(),
            overlay: false,
            saved_high_scores: high_scores,
            settings,
            menu: None,
            resize_window: true,
        }
    }

    /// The session the app plays.
    pub(crate) fn session(&self) -> &Session {
        &self.session
    }

    /// Puts `settings`, as the settings screen closed with them, in
    /// force: the next game plays by their rules, the window takes their
    /// size, and they are written to `storage` and flushed. Does nothing
    /// when they are the settings in force.
    pub fn apply_settings(&mut self, settings: Settings, storage: Option<&mut dyn Storage>) {
        if settings == self.settings {
            return;
        }
        self.resize_window |= settings.scale != self.settings.scale;
        self.settings = settings;
        self.session.set_rules(settings.rules());
        if let Some(storage) = storage {
            persist::save_settings(storage, &settings);
            storage.flush();
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
    /// Samples the keyboard and runs as many ticks as the frame's time
    /// owes, of the settings screen while it is open, of the session
    /// otherwise. Returns the settings the screen closed with, if it did.
    fn advance(&mut self, ctx: &egui::Context) -> Option<Settings> {
        let keys = self
            .menu
            .map_or(self.settings.keys, |menu| menu.settings().keys);
        let (sample, overlay_toggled, settings_opened, pressed, dt) = ctx.input(|i| {
            let toggled = input::overlay_toggled(i);
            let opened = input::settings_opened(i);
            (
                input::sample(i, &keys),
                toggled,
                opened,
                input::pressed_keys(i),
                i.stable_dt,
            )
        });
        self.overlay ^= overlay_toggled;
        match &mut self.menu {
            Some(menu) if menu.is_capturing() => {
                pressed.into_iter().any(|key| menu.capture(key));
            }
            Some(_) => {}
            None if settings_opened && self.session.state() == GameState::Title => {
                self.menu = Some(SettingsMenu::new(self.settings));
            }
            None => {}
        }
        self.input.record(sample);
        (0..self.timestep.advance(dt)).find_map(|_| self.tick())
    }

    /// Runs one tick of the settings screen while it is open, of the
    /// session otherwise; returns the settings the screen closed with,
    /// if it did.
    fn tick(&mut self) -> Option<Settings> {
        let tick_input = self.input.next_tick();
        let Some(menu) = &mut self.menu else {
            self.session.tick(&tick_input);
            return None;
        };
        let closed = menu.tick(&tick_input);
        if closed.is_some() {
            self.menu = None;
        }
        closed
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
        if let Some(settings) = self.advance(ui.ctx()) {
            let storage = frame
                .storage_mut()
                .map(|storage| storage as &mut dyn Storage);
            self.apply_settings(settings, storage);
        }
        self.save_new_high_scores(frame);
        if std::mem::take(&mut self.resize_window) {
            let size = self.settings.scale.window_size();
            ui.ctx()
                .send_viewport_cmd(egui::ViewportCommand::InnerSize(size));
        }
        egui::CentralPanel::default()
            .frame(playfield::panel_frame())
            .show(ui, |ui| {
                let session = self.session();
                let game = session.game();
                let hud = hud::hud(game, session.high_score());
                let screen = match &self.menu {
                    Some(menu) => Some(settings_screen(menu)),
                    None => screen::screen(session, &self.settings.keys),
                };
                playfield::show(ui, game, &hud, screen.as_ref(), self.overlay);
            });
        // The game moves on its own, so every frame is followed by another.
        ui.ctx().request_repaint();
    }

    fn save(&mut self, storage: &mut dyn Storage) {
        persist::save_high_scores(storage, self.session.high_scores());
        persist::save_settings(storage, &self.settings);
    }
}
