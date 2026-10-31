//! The debug overlay: the claimed share of the board, the lives left and
//! the tick count.

use eframe::egui::{FontId, Painter, Rect, vec2};
use qix_core::Game;

use crate::palette;

/// Size of the overlay's text, in points.
const FONT_SIZE: f32 = 14.0;

/// Gap between the playfield's top left corner and the overlay, in points.
const MARGIN: f32 = 8.0;

/// Space between the overlay's text and the edge of its background, in
/// points.
const PADDING: f32 = 4.0;

/// Corner radius of the overlay's background, in points.
const CORNER_RADIUS: f32 = 4.0;

/// Converts a fraction to a percentage.
const PERCENT: f32 = 100.0;

/// The overlay's lines for `game`.
pub fn text(game: &Game) -> String {
    let claimed = game.board().claimed_fraction() * PERCENT;
    format!(
        "claimed {claimed:.1}%\nlives {}\ntick {}",
        game.lives(),
        game.ticks()
    )
}

/// Paints the overlay in the top left corner of `field`, the playfield on
/// screen.
pub fn paint(painter: &Painter, field: Rect, game: &Game) {
    let galley = painter.layout_no_wrap(
        text(game),
        FontId::monospace(FONT_SIZE),
        palette::OVERLAY_TEXT,
    );
    let pos = field.min + vec2(MARGIN + PADDING, MARGIN + PADDING);
    let background = Rect::from_min_size(pos, galley.size()).expand(PADDING);
    painter.rect_filled(background, CORNER_RADIUS, palette::OVERLAY_BACKGROUND);
    painter.galley(pos, galley, palette::OVERLAY_TEXT);
}
