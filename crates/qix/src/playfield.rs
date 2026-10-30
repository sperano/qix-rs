//! The letterboxed playfield: where it sits in the window and how it is
//! painted.

use eframe::egui::{
    self, Color32, Frame, Pos2, Rect, Sense, Stroke, StrokeKind, emath::RectTransform, vec2,
};
use qix_core::tuning::{PLAYFIELD_HEIGHT, PLAYFIELD_WIDTH};

const WINDOW_BACKGROUND: Color32 = Color32::BLACK;
const PLAYFIELD_BACKGROUND: Color32 = Color32::from_rgb(16, 16, 40);
const BORDER_COLOR: Color32 = Color32::WHITE;

/// Width of the playfield border, in points.
const BORDER_WIDTH: f32 = 2.0;

/// Gap between the window edge and the playfield, in points; wider than
/// the border, which is drawn outside the playfield.
const WINDOW_MARGIN: f32 = 8.0;

/// The frame of the panel holding the playfield.
pub fn panel_frame() -> Frame {
    Frame::NONE
        .fill(WINDOW_BACKGROUND)
        .inner_margin(WINDOW_MARGIN)
}

/// The playfield in its own units: one unit per cell, origin at the top
/// left.
pub fn playfield_units() -> Rect {
    Rect::from_min_size(
        Pos2::ZERO,
        vec2(f32::from(PLAYFIELD_WIDTH), f32::from(PLAYFIELD_HEIGHT)),
    )
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

/// Fills `ui` with the playfield: its background and border.
pub fn show(ui: &mut egui::Ui) {
    let (response, painter) = ui.allocate_painter(ui.available_size(), Sense::hover());
    let field = to_screen(response.rect).transform_rect(playfield_units());
    painter.rect_filled(field, 0.0, PLAYFIELD_BACKGROUND);
    painter.rect_stroke(
        field,
        0.0,
        Stroke::new(BORDER_WIDTH, BORDER_COLOR),
        StrokeKind::Outside,
    );
}
