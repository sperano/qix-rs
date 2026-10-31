//! The enemies: the Qix and its trail, the Sparx and the fuse.
//!
//! Sizes are in screen points, so the enemies keep their look at every
//! window size; positions go through [`paint`]'s transform like the
//! board's.

use std::f32::consts::FRAC_PI_4;

use eframe::egui::{Color32, Painter, Pos2, Shape, Stroke, emath::RectTransform, pos2, vec2};
use qix_core::{Game, Qix, Sparx, Vec2};

use crate::palette;
use crate::render::board::screen;

/// Width of each segment of the Qix's trail, in points.
const QIX_WIDTH: f32 = 1.5;

/// Half the length of a spark's long rays, in points.
const SPARK_LONG_RAY: f32 = 5.0;

/// Half the length of a spark's short rays, in points.
const SPARK_SHORT_RAY: f32 = 2.5;

/// Width of a spark's rays, in points.
const SPARK_WIDTH: f32 = 1.5;

/// The rays through a spark's centre: horizontal, diagonal, vertical and
/// the other diagonal, every other one long.
pub const SPARK_RAYS: usize = 4;

/// Ticks between two flickers of a spark, when its long and short rays
/// swap.
pub const SPARK_FLICKER_TICKS: u64 = 4;

/// Radius of the fuse's bright dot, in points.
pub const FUSE_RADIUS: f32 = 2.5;

/// Radius of the outermost ring of the fuse's glow, in points.
const FUSE_GLOW_RADIUS: f32 = 7.0;

/// Translucent rings the fuse's glow is made of: they stack towards the
/// dot, so the glow fades outwards.
pub const FUSE_GLOW_RINGS: usize = 3;

/// Opacity of each ring of the fuse's glow.
const FUSE_GLOW_OPACITY: f32 = 0.3;

/// Paints `game`'s enemies through `to_screen`: each Qix's trail, then the
/// Sparx and the fuse on top.
pub fn paint(painter: &Painter, to_screen: RectTransform, game: &Game) {
    for qix in game.qix() {
        paint_qix(painter, to_screen, qix, game.ticks());
    }
    for sparx in game.sparx() {
        paint_sparx(painter, to_screen, sparx, game.ticks());
    }
    let fuse_point = game
        .stix()
        .zip(game.fuse())
        .and_then(|(stix, fuse)| stix.points().get(fuse.pos).copied());
    if let Some(point) = fuse_point {
        painter.extend(fuse_shapes(screen(to_screen, point)));
    }
}

/// The colours of a trail of `len` segments after `ticks` ticks, oldest
/// first: one step through [`palette::QIX_TRAIL`] per segment, so a
/// segment keeps its colour as long as it stays in the trail.
///
/// The Qix records one segment per tick, so its newest segment is that of
/// tick `ticks`.
pub fn trail_colours(len: usize, ticks: u64) -> impl Iterator<Item = Color32> {
    let cycle = palette::QIX_TRAIL.len();
    // Each fits in `usize`: a remainder of a division by a `usize`.
    let newest = (ticks % cycle as u64) as usize;
    (0..len).map(move |i| {
        let age = (len - 1 - i) % cycle;
        palette::QIX_TRAIL[(newest + cycle - age) % cycle]
    })
}

/// The ends of a spark's rays, each through `center`; which rays are long
/// alternates every [`SPARK_FLICKER_TICKS`].
pub fn spark_rays(center: Pos2, ticks: u64) -> [[Pos2; 2]; SPARK_RAYS] {
    let flicker = usize::from(!(ticks / SPARK_FLICKER_TICKS).is_multiple_of(2));
    std::array::from_fn(|ray| {
        let half = if (ray + flicker) % 2 == 0 {
            SPARK_LONG_RAY
        } else {
            SPARK_SHORT_RAY
        };
        let dir = Vec2::from_angle(ray as f32 * FRAC_PI_4);
        let offset = vec2(dir.x, dir.y) * half;
        [center - offset, center + offset]
    })
}

/// The fuse at `center`: the rings of its glow, outermost first, then its
/// bright dot.
pub fn fuse_shapes(center: Pos2) -> Vec<Shape> {
    let glow = palette::FUSE_GLOW.gamma_multiply(FUSE_GLOW_OPACITY);
    let step = (FUSE_GLOW_RADIUS - FUSE_RADIUS) / FUSE_GLOW_RINGS as f32;
    (0..FUSE_GLOW_RINGS)
        .map(|ring| {
            let radius = FUSE_GLOW_RADIUS - step * ring as f32;
            Shape::circle_filled(center, radius, glow)
        })
        .chain([Shape::circle_filled(center, FUSE_RADIUS, palette::FUSE)])
        .collect()
}

/// Paints the Qix's trail, oldest segment first, so the current segment
/// lies on top.
fn paint_qix(painter: &Painter, to_screen: RectTransform, qix: &Qix, ticks: u64) {
    let trail = qix.trail();
    let colours = trail_colours(trail.len(), ticks);
    painter.extend(trail.iter().zip(colours).map(|(&(a, b), colour)| {
        let ends = [a, b].map(|end| to_screen.transform_pos(pos2(end.x, end.y)));
        Shape::line_segment(ends, Stroke::new(QIX_WIDTH, colour))
    }));
}

/// Paints a spark on `sparx`'s lattice point, in the colour of its kind.
fn paint_sparx(painter: &Painter, to_screen: RectTransform, sparx: &Sparx, ticks: u64) {
    let stroke = Stroke::new(SPARK_WIDTH, palette::sparx(sparx.kind));
    let rays = spark_rays(screen(to_screen, sparx.pos), ticks);
    painter.extend(rays.map(|ends| Shape::line_segment(ends, stroke)));
}
