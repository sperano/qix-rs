//! The colours the game is painted with.

use eframe::egui::Color32;
use qix_core::{DrawKind, SparxKind};

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

/// The colours the Qix's trail cycles through, one per segment.
pub const QIX_TRAIL: [Color32; 8] = [
    Color32::from_rgb(255, 70, 70),
    Color32::from_rgb(255, 160, 40),
    Color32::from_rgb(240, 240, 60),
    Color32::from_rgb(80, 230, 80),
    Color32::from_rgb(60, 230, 230),
    Color32::from_rgb(80, 120, 255),
    Color32::from_rgb(170, 90, 255),
    Color32::from_rgb(255, 80, 210),
];

/// A Sparx walking the border.
pub const SPARX: Color32 = Color32::from_rgb(255, 90, 90);

/// A Super Sparx.
pub const SUPER_SPARX: Color32 = Color32::from_rgb(90, 255, 140);

/// The fuse's burning point.
pub const FUSE: Color32 = Color32::from_rgb(255, 255, 230);

/// The glow around the fuse, before it is made translucent.
pub const FUSE_GLOW: Color32 = Color32::from_rgb(255, 170, 40);

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

/// The colour of a Sparx of `kind`.
pub const fn sparx(kind: SparxKind) -> Color32 {
    match kind {
        SparxKind::Normal => SPARX,
        SparxKind::Super => SUPER_SPARX,
    }
}
