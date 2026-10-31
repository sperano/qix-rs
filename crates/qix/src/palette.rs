//! The colours the game is painted with.

use eframe::egui::Color32;
use qix_core::DrawKind;

/// Around the letterboxed playfield.
pub const WINDOW_BACKGROUND: Color32 = Color32::BLACK;

/// The free area of the playfield.
pub const PLAYFIELD_BACKGROUND: Color32 = Color32::from_rgb(16, 16, 40);

/// The border the marker and the Sparx travel.
pub const BORDER: Color32 = Color32::WHITE;

/// Cells claimed by a fast draw.
pub const FAST_FILL: Color32 = Color32::from_rgb(40, 90, 200);

/// Cells claimed by a slow draw.
pub const SLOW_FILL: Color32 = Color32::from_rgb(190, 70, 40);

/// Segments of the stix drawn fast.
pub const FAST_STIX: Color32 = Color32::from_rgb(130, 180, 255);

/// Segments of the stix drawn slow.
pub const SLOW_STIX: Color32 = Color32::from_rgb(255, 140, 100);

/// The player's marker.
pub const MARKER: Color32 = Color32::from_rgb(255, 220, 60);

/// Text of the debug overlay.
pub const OVERLAY_TEXT: Color32 = Color32::WHITE;

/// Behind the debug overlay's text, translucent so the board shows
/// through.
pub const OVERLAY_BACKGROUND: Color32 = Color32::from_black_alpha(160);

/// The fill of cells claimed with `kind`.
pub const fn fill(kind: DrawKind) -> Color32 {
    match kind {
        DrawKind::Fast => FAST_FILL,
        DrawKind::Slow => SLOW_FILL,
    }
}

/// The colour of stix segments drawn with `kind`.
pub const fn stix(kind: DrawKind) -> Color32 {
    match kind {
        DrawKind::Fast => FAST_STIX,
        DrawKind::Slow => SLOW_STIX,
    }
}
