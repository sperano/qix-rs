//! Desktop front end of the Qix clone: an eframe window around `qix_core`.

use eframe::egui;

const APP_NAME: &str = "Qix";

/// Placeholder until the game shell lands: an empty window.
struct QixApp;

impl eframe::App for QixApp {
    fn ui(&mut self, _ui: &mut egui::Ui, _frame: &mut eframe::Frame) {}
}

fn main() -> eframe::Result {
    eframe::run_native(
        APP_NAME,
        eframe::NativeOptions::default(),
        Box::new(|_cc| Ok(Box::new(QixApp))),
    )
}
