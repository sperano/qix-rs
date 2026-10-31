//! Desktop front end of the Qix clone: an eframe window around `qix_core`.

mod app;
mod input;
mod numbers;
mod palette;
mod playfield;
mod render;
mod timestep;

#[cfg(test)]
mod input_test;
#[cfg(test)]
mod numbers_test;
#[cfg(test)]
mod playfield_test;
#[cfg(test)]
mod timestep_test;

use eframe::egui;

const APP_NAME: &str = "Qix";

/// Initial inner size of the window in points, at the ratio of the
/// playfield and the HUD above it.
const DEFAULT_WINDOW_SIZE: [f32; 2] = [960.0, 792.0];

/// Smallest inner size the window can be resized to, in points.
const MIN_WINDOW_SIZE: [f32; 2] = [320.0, 240.0];

fn main() -> eframe::Result {
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
        Box::new(|_cc| Ok(Box::<app::QixApp>::default())),
    )
}
