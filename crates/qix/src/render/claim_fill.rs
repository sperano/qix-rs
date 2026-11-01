//! The fill animation of a claim, over the claimed cells: see
//! [`crate::claim_fill`].

use eframe::egui::{Color32, Mesh, Painter, emath::RectTransform};
use qix_core::DrawKind;

use crate::palette;
use crate::render::board::CellRun;

/// The colour of cells claimed with `kind` once their animation is
/// `progress` through, from 0 at the claim to 1 at its end: from
/// [`palette::CLAIM_FLASH`] to the colour of their fill, fast at first
/// and slowing down towards the fill.
pub fn fill_colour(kind: DrawKind, progress: f32) -> Color32 {
    let t = progress.clamp(0.0, 1.0);
    let eased = 1.0 - (1.0 - t) * (1.0 - t);
    palette::CLAIM_FLASH.lerp_to_gamma(palette::fill(kind), eased)
}

/// Paints the cells of `runs` through `to_screen`, a claim's animation
/// `progress` through, over the fill the board paints them with.
pub fn paint(painter: &Painter, to_screen: RectTransform, runs: &[CellRun], progress: f32) {
    let mut mesh = Mesh::default();
    for run in runs {
        mesh.add_colored_rect(
            to_screen.transform_rect(run.rect()),
            fill_colour(run.kind, progress),
        );
    }
    painter.add(mesh);
}
