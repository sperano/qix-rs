//! The letterboxed playfield: where it sits in the window and how it is
//! painted.

use eframe::egui::{self, Frame, Pos2, Rect, Sense, emath::RectTransform, vec2};
use qix_core::Game;
use qix_core::tuning::{BOARD_H, BOARD_W};

use crate::palette;
use crate::render;
use crate::render::screen::Screen;

/// Gap between the window edge and the playfield, in points; wider than
/// the half of the border lines that lies outside the playfield.
const WINDOW_MARGIN: f32 = 8.0;

/// The frame of the panel holding the playfield.
pub fn panel_frame() -> Frame {
    Frame::NONE
        .fill(palette::WINDOW_BACKGROUND)
        .inner_margin(WINDOW_MARGIN)
}

/// The playfield in its own units: one unit per cell, origin at the top
/// left.
pub fn playfield_units() -> Rect {
    Rect::from_min_size(Pos2::ZERO, vec2(f32::from(BOARD_W), f32::from(BOARD_H)))
}

/// The largest rectangle with the playfield's aspect ratio that fits in
/// `available`, centred in it.
pub fn letterbox(available: Rect) -> Rect {
    let size = playfield_units().size();
    let scale = (available.width() / size.x)
        .min(available.height() / size.y)
        .max(0.0);
    Rect::from_center_size(available.center(), size * scale)
}

/// Maps playfield units to the letterboxed playfield inside `available`.
pub fn to_screen(available: Rect) -> RectTransform {
    RectTransform::from_to(playfield_units(), letterbox(available))
}

/// Fills `ui` with the playfield: its background, `game`'s board and its
/// enemies, then `screen` over them, if any, and the debug overlay on top
/// when `overlay` is set.
pub fn show(ui: &mut egui::Ui, game: &Game, screen: Option<&Screen>, overlay: bool) {
    let (response, painter) = ui.allocate_painter(ui.available_size(), Sense::hover());
    let to_screen = to_screen(response.rect);
    let field = to_screen.transform_rect(playfield_units());
    painter.rect_filled(field, 0.0, palette::PLAYFIELD_BACKGROUND);
    render::board::paint(&painter, to_screen, game);
    render::enemies::paint(&painter, to_screen, game);
    if let Some(screen) = screen {
        render::screen::paint(&painter, field, screen);
    }
    if overlay {
        render::overlay::paint(&painter, field, game);
    }
}
