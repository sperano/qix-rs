//! The claimed cells, drawn into a texture of one texel per cell that is
//! redrawn only when the board changes: once painted, a claimed area costs
//! one textured rectangle per frame, however many runs its rows hold.

use std::fmt;

use eframe::egui::{
    Color32, ColorImage, Context, Painter, Rect, TextureHandle, TextureId, TextureOptions,
    emath::RectTransform, pos2, vec2,
};
use qix_core::{Board, Cell};

use crate::palette;

/// The name of the texture, for egui's debugging.
const TEXTURE_NAME: &str = "claimed cells";

/// Nearest filtering keeps each cell a sharp square however the playfield
/// is scaled.
const TEXTURE_OPTIONS: TextureOptions = TextureOptions::NEAREST;

/// The texture of the claimed cells of the board last painted.
#[derive(Default)]
pub struct ClaimedTexture {
    /// The board the texture was last drawn from, and the texture.
    cached: Option<(Board, TextureHandle)>,
}

impl ClaimedTexture {
    /// The texture of `board`'s claimed cells, loaded into `ctx` on the
    /// first call and redrawn when `board` differs from the board it was
    /// last drawn from.
    pub fn texture(&mut self, ctx: &Context, board: &Board) -> TextureId {
        match &mut self.cached {
            Some((seen, handle)) => {
                if seen != board {
                    handle.set(image(board), TEXTURE_OPTIONS);
                    seen.clone_from(board);
                }
                handle.id()
            }
            None => {
                let handle = ctx.load_texture(TEXTURE_NAME, image(board), TEXTURE_OPTIONS);
                let id = handle.id();
                self.cached = Some((board.clone(), handle));
                id
            }
        }
    }
}

impl fmt::Debug for ClaimedTexture {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let id = self.cached.as_ref().map(|(_, handle)| handle.id());
        f.debug_struct("ClaimedTexture")
            .field("texture", &id)
            .finish_non_exhaustive()
    }
}

/// `board` as an image of one texel per cell, row by row from the top:
/// each claimed cell in the fill of its kind, the free ones transparent.
pub fn image(board: &Board) -> ColorImage {
    let (width, height) = (board.width(), board.height());
    let pixels = (0..height)
        .flat_map(|y| (0..width).map(move |x| (x, y)))
        .map(|(x, y)| match board.cell(x.into(), y.into()) {
            Cell::Claimed(kind) => palette::fill(kind),
            Cell::Free => Color32::TRANSPARENT,
        })
        .collect();
    ColorImage::new([width.into(), height.into()], pixels)
}

/// Paints `texture`, the claimed cells of a `width x height` board, over
/// the board through `to_screen`.
pub fn paint(
    painter: &Painter,
    to_screen: RectTransform,
    texture: TextureId,
    (width, height): (u16, u16),
) {
    let cells = Rect::from_min_size(pos2(0.0, 0.0), vec2(width.into(), height.into()));
    let uv = Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0));
    painter.image(texture, to_screen.transform_rect(cells), uv, Color32::WHITE);
}
