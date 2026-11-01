//! The desktop entry point: a native window.

use eframe::egui;

use crate::app;
use crate::settings::{BASE_WINDOW_SIZE, SCALE_DEFAULT};

const APP_NAME: &str = "Qix";

/// Initial inner size of the window in points, at the default scale,
/// until the first frame applies the scale of the settings.
pub const DEFAULT_WINDOW_SIZE: [f32; 2] = [
    BASE_WINDOW_SIZE[0] * SCALE_DEFAULT,
    BASE_WINDOW_SIZE[1] * SCALE_DEFAULT,
];

/// Smallest inner size the window can be resized to, in points.
pub const MIN_WINDOW_SIZE: [f32; 2] = [320.0, 240.0];

/// Opens the window and runs the game until it closes.
pub fn run() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title(APP_NAME)
            .with_inner_size(DEFAULT_WINDOW_SIZE)
            .with_min_inner_size(MIN_WINDOW_SIZE),
        ..Default::default()
    };
    eframe::run_native(
        APP_NAME,
        options,
        Box::new(|cc| Ok(Box::new(app::QixApp::new(cc.storage)))),
    )
}
