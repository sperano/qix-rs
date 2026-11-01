//! Pixel-art sprites: masks of characters, one per sprite pixel, painted
//! as one mesh of squares.
//!
//! A mask has an odd width and height, so its middle pixel lies on the
//! lattice point the sprite is centred on. Sizes are in screen points, so
//! sprites keep their look at every window size.

use eframe::egui::{Color32, Mesh, Pos2, Rect, vec2};
use qix_core::SparxKind;

use crate::palette;

/// The rows of a sprite, top first, one character per pixel. The
/// characters are colour roles that the caller maps to colours;
/// [`TRANSPARENT`] is the only one with a meaning of its own.
pub type Mask = &'static [&'static str];

/// Size of one sprite pixel in the playfield, in points.
pub const PIXEL: f32 = 1.25;

/// The role of a pixel that is not painted.
pub const TRANSPARENT: char = '.';

/// The marker's outline.
pub const OUTLINE: char = 'O';

/// The marker's inside.
pub const INNER: char = 'I';

/// The marker's highlight, on its upper left edge.
pub const HIGHLIGHT: char = 'H';

/// The arms of a spark.
pub const BODY: char = 'B';

/// The bright heart of a spark.
pub const CORE: char = 'C';

/// The marker: a diamond with an outline and a highlight.
pub const MARKER: Mask = &[
    "....O....",
    "...OIO...",
    "..OHIIO..",
    ".OHHIIIO.",
    "OIIIIIIIO",
    ".OIIIIIO.",
    "..OIIIO..",
    "...OIO...",
    "....O....",
];

/// The two frames of a spark: the plus-shaped arms, then the diagonal
/// ones, around the same core.
pub const SPARK_FRAMES: [Mask; 2] = [
    &[
        "....B....",
        "....B....",
        "....B....",
        "...BCB...",
        "BBBCCCBBB",
        "...BCB...",
        "....B....",
        "....B....",
        "....B....",
    ],
    &[
        "B.......B",
        ".B.....B.",
        "..B...B..",
        "...BCB...",
        "...CCC...",
        "...BCB...",
        "..B...B..",
        ".B.....B.",
        "B.......B",
    ],
];

/// The width and height of `mask`, in sprite pixels.
pub fn mask_size(mask: Mask) -> (usize, usize) {
    let width = mask.first().map_or(0, |row| row.chars().count());
    (width, mask.len())
}

/// `mask` as a mesh of one `pixel`-point square per painted pixel,
/// centred on `centre`. `colour_of` gives the colour of a role, or `None`
/// to leave its pixels unpainted like [`TRANSPARENT`] ones.
///
/// A mesh is not anti-aliased, so squares that share an edge leave no seam.
pub fn mesh(
    mask: Mask,
    centre: Pos2,
    pixel: f32,
    colour_of: impl Fn(char) -> Option<Color32>,
) -> Mesh {
    let (width, height) = mask_size(mask);
    let origin = centre - vec2(width as f32, height as f32) * (pixel / 2.0);
    let mut mesh = Mesh::default();
    for (row, line) in mask.iter().enumerate() {
        for (col, role) in line.chars().enumerate() {
            let Some(colour) = colour_of(role).filter(|_| role != TRANSPARENT) else {
                continue;
            };
            let min = origin + vec2(col as f32, row as f32) * pixel;
            mesh.add_colored_rect(Rect::from_min_size(min, vec2(pixel, pixel)), colour);
        }
    }
    mesh
}

/// The marker centred on `centre` with `pixel`-point pixels, its inside
/// painted `inner`.
pub fn marker(centre: Pos2, pixel: f32, inner: Color32) -> Mesh {
    let colour_of = |role| match role {
        OUTLINE => Some(palette::MARKER_OUTLINE),
        INNER => Some(inner),
        HIGHLIGHT => Some(palette::MARKER_HIGHLIGHT),
        _ => None,
    };
    mesh(MARKER, centre, pixel, colour_of)
}

/// The spark showing `mask`, one of [`SPARK_FRAMES`], centred on `centre`
/// in the colours of a Sparx of `kind`.
pub fn spark(mask: Mask, centre: Pos2, kind: SparxKind) -> Mesh {
    let colour_of = |role| match role {
        BODY => Some(palette::sparx(kind)),
        CORE => Some(palette::sparx_core(kind)),
        _ => None,
    };
    mesh(mask, centre, PIXEL, colour_of)
}
