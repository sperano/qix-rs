//! What the application keeps between runs, in eframe's [`Storage`].

use eframe::Storage;
use qix_core::HighScores;

use crate::settings::Settings;

/// The [`Storage`] key of the high score table.
pub const HIGH_SCORES_KEY: &str = "high_scores";

/// The [`Storage`] key of the settings.
pub const SETTINGS_KEY: &str = "settings";

/// The high score table kept in `storage`; an empty one when there is no
/// storage, no table in it, or one that does not decode.
pub fn load_high_scores(storage: Option<&dyn Storage>) -> HighScores {
    storage
        .and_then(|storage| eframe::get_value(storage, HIGH_SCORES_KEY))
        .unwrap_or_default()
}

/// Writes `high_scores` to `storage`, in place of the table kept there.
pub fn save_high_scores(storage: &mut dyn Storage, high_scores: &HighScores) {
    eframe::set_value(storage, HIGH_SCORES_KEY, high_scores);
}

/// The settings kept in `storage`; the default ones when there is no
/// storage, no settings in it, or ones that do not decode (see
/// [`Settings`] for the fields that load on their own).
pub fn load_settings(storage: Option<&dyn Storage>) -> Settings {
    storage
        .and_then(|storage| eframe::get_value(storage, SETTINGS_KEY))
        .unwrap_or_default()
}

/// Writes `settings` to `storage`, in place of the ones kept there.
pub fn save_settings(storage: &mut dyn Storage, settings: &Settings) {
    eframe::set_value(storage, SETTINGS_KEY, settings);
}
