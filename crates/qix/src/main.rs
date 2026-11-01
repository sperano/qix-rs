//! Desktop front end of the Qix clone: an eframe window around `qix_core`.

mod app;
mod audio;
mod bindings;
mod claim_fill;
mod input;
mod numbers;
mod palette;
mod persist;
mod playfield;
mod render;
mod settings;
mod settings_menu;
mod sound;
#[cfg_attr(
    not(feature = "audio"),
    allow(dead_code, reason = "only the audio feature plays the waveforms")
)]
mod synth;
mod timestep;

#[cfg(test)]
mod app_test;
#[cfg(test)]
mod bindings_test;
#[cfg(test)]
mod claim_fill_test;
#[cfg(test)]
mod input_test;
#[cfg(test)]
mod numbers_test;
#[cfg(test)]
mod persist_test;
#[cfg(test)]
mod playfield_test;
#[cfg(test)]
mod settings_menu_test;
#[cfg(test)]
mod settings_test;
#[cfg(test)]
mod sound_test;
#[cfg(test)]
mod synth_test;
#[cfg(test)]
mod timestep_test;

use eframe::egui;

use crate::settings::{BASE_WINDOW_SIZE, SCALE_DEFAULT};

const APP_NAME: &str = "Qix";

/// Initial inner size of the window in points, at the default scale,
/// until the first frame applies the scale of the settings.
const DEFAULT_WINDOW_SIZE: [f32; 2] = [
    BASE_WINDOW_SIZE[0] * SCALE_DEFAULT,
    BASE_WINDOW_SIZE[1] * SCALE_DEFAULT,
];

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
        Box::new(|cc| Ok(Box::new(app::QixApp::new(cc.storage)))),
    )
}
