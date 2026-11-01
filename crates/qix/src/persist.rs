//! What the application keeps between runs, in eframe's [`Storage`].

use eframe::Storage;
use qix_core::HighScores;

/// The [`Storage`] key of the high score table.
pub const HIGH_SCORES_KEY: &str = "high_scores";

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
