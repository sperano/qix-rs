//! Front end of the Qix clone: an eframe window around `qix_core` on the
//! desktop, a canvas on the web.

mod app;
mod audio;
mod bindings;
mod claim_fill;
mod input;
#[cfg(not(target_arch = "wasm32"))]
mod native;
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
#[cfg(target_arch = "wasm32")]
mod web;

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

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result {
    native::run()
}

#[cfg(target_arch = "wasm32")]
fn main() {
    web::start();
}
