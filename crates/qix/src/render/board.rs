//! The board: claimed cells, the border, the stix and the marker.
//!
//! Positions are in playfield units (one unit per cell, lattice point
//! `(x, y)` at `(x, y)`) until [`paint`] maps them to the screen.

use std::ops::Range;

use eframe::egui::{
    Color32, Painter, Pos2, Rect, Shape, Stroke, TextureId, emath::RectTransform, pos2, vec2,
};
use qix_core::{Board, Cell, Dir, DrawKind, Game, Point, Stix};

use crate::palette;
use crate::render::{claim_fill, claimed, sprite};

/// Width of the border lines, in points.
const BORDER_WIDTH: f32 = 2.0;

/// Width of the stix, in points.
const STIX_WIDTH: f32 = 2.0;

/// Game ticks between two flickers of the stix being drawn, when it swaps
/// between the colour of its kind and [`palette::STIX_FLICKER`].
pub const STIX_FLICKER_TICKS: u64 = 3;

/// A horizontal run of claimed cells of the same kind in one row.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CellRun {
    pub row: u16,
    pub cols: Range<u16>,
    pub kind: DrawKind,
}

impl CellRun {
    /// The cells the run covers, in playfield units.
    pub fn rect(&self) -> Rect {
        let min = pos2(f32::from(self.cols.start), f32::from(self.row));
        let width = f32::from(self.cols.end - self.cols.start);
        Rect::from_min_size(min, vec2(width, 1.0))
    }
}

/// Paints `game`'s board through `to_screen`: the claimed cells from
/// `claimed_texture`, the texture of its board's claimed cells (see
/// [`claimed::ClaimedTexture`]), and over them `claim`, the cells of the
/// claim being animated and how far its animation is, if any, then the
/// border, the stix and the marker on top.
pub fn paint(
    painter: &Painter,
    to_screen: RectTransform,
    game: &Game,
    claimed_texture: TextureId,
    claim: Option<(&[CellRun], f32)>,
) {
    let board = game.board();
    claimed::paint(
        painter,
        to_screen,
        claimed_texture,
        (board.width(), board.height()),
    );
    if let Some((runs, progress)) = claim {
        claim_fill::paint(painter, to_screen, runs, progress);
    }
    let border = Stroke::new(BORDER_WIDTH, palette::BORDER);
    painter.extend(border_segments(board).into_iter().map(|(from, to)| {
        let ends = [screen(to_screen, from), screen(to_screen, to)];
        Shape::line_segment(square_capped(ends, BORDER_WIDTH), border)
    }));
    if let Some(stix) = game.stix() {
        paint_stix(painter, to_screen, stix, game.ticks());
    }
    let marker = screen(to_screen, game.marker().pos);
    let inner = marker_inner(game.stix());
    painter.add(sprite::marker(marker, sprite::PIXEL, inner));
}

/// Every claimed cell of `board` as maximal runs along its rows, top row
/// first and left to right within a row.
pub fn claimed_runs(board: &Board) -> Vec<CellRun> {
    (0..board.height())
        .flat_map(|row| {
            let kind_at = move |col: u16| match board.cell(col.into(), row.into()) {
                Cell::Claimed(kind) => Some(kind),
                Cell::Free => None,
            };
            merge_runs(board.width(), kind_at)
                .into_iter()
                .map(move |(cols, kind)| CellRun { row, cols, kind })
        })
        .collect()
}

/// The border of `board` as maximal straight segments between lattice
/// points: the horizontal ones top row first, then the vertical ones left
/// column first.
pub fn border_segments(board: &Board) -> Vec<(Point, Point)> {
    let (width, height) = (board.width(), board.height());
    let horizontal = (0..=height).flat_map(|y| {
        let border_at = move |x| {
            board
                .is_border_edge(Point::new(x, y), Dir::Right)
                .then_some(())
        };
        merge_runs(width, border_at)
            .into_iter()
            .map(move |(xs, ())| (Point::new(xs.start, y), Point::new(xs.end, y)))
    });
    let vertical = (0..=width).flat_map(|x| {
        let border_at = move |y| {
            board
                .is_border_edge(Point::new(x, y), Dir::Down)
                .then_some(())
        };
        merge_runs(height, border_at)
            .into_iter()
            .map(move |(ys, ())| (Point::new(x, ys.start), Point::new(x, ys.end)))
    });
    horizontal.chain(vertical).collect()
}

/// The stix as polylines of consecutive segments drawn with the same
/// kind, from the start to the head; consecutive polylines share their
/// meeting point. A stix with no segment has none.
pub fn stix_runs(stix: &Stix) -> Vec<(DrawKind, &[Point])> {
    let points = stix.points();
    let mut first = 0;
    stix.kinds()
        .chunk_by(|a, b| a == b)
        .map(|kinds| {
            let last = first + kinds.len();
            let run = (kinds[0], &points[first..=last]);
            first = last;
            run
        })
        .collect()
}

/// The colour of the stix's segments drawn with `kind` after `ticks` game
/// ticks: it alternates with [`palette::STIX_FLICKER`] every
/// [`STIX_FLICKER_TICKS`], starting with the colour of its kind.
pub fn stix_colour(kind: DrawKind, ticks: u64) -> Color32 {
    if (ticks / STIX_FLICKER_TICKS).is_multiple_of(2) {
        palette::stix(kind)
    } else {
        palette::STIX_FLICKER
    }
}

/// The colour of the marker's inside: that of the kind of the stix's last
/// segment while it draws one, plain otherwise.
pub fn marker_inner(stix: Option<&Stix>) -> Color32 {
    stix.and_then(|stix| stix.kinds().last())
        .map_or(palette::MARKER, |&kind| palette::stix(kind))
}

/// The ends of a line `width` wide from `ends[0]` to `ends[1]`, each pushed
/// out by half the width. egui ends an open line flush with its end
/// points, so where two border lines meet at a corner the corner's outer
/// square would stay unpainted; the extension squares it off.
pub fn square_capped([from, to]: [Pos2; 2], width: f32) -> [Pos2; 2] {
    let extension = (to - from).normalized() * (width / 2.0);
    [from - extension, to + extension]
}

/// Paints the stix, each run of segments in the colour of its kind as
/// [`stix_colour`] flickers it after `ticks` game ticks.
fn paint_stix(painter: &Painter, to_screen: RectTransform, stix: &Stix, ticks: u64) {
    for (kind, points) in stix_runs(stix) {
        let points = points.iter().map(|&p| screen(to_screen, p)).collect();
        painter.line(points, Stroke::new(STIX_WIDTH, stix_colour(kind, ticks)));
    }
}

/// The screen position of lattice point `point`.
pub fn screen(to_screen: RectTransform, point: Point) -> Pos2 {
    to_screen.transform_pos(pos2(f32::from(point.x), f32::from(point.y)))
}

/// The maximal runs of equal values among `value_at(0..len)`, skipping
/// the indices where it is `None`, in increasing order.
fn merge_runs<T: Copy + PartialEq>(
    len: u16,
    value_at: impl Fn(u16) -> Option<T>,
) -> Vec<(Range<u16>, T)> {
    let mut runs: Vec<(Range<u16>, T)> = Vec::new();
    for i in 0..len {
        let Some(value) = value_at(i) else { continue };
        match runs.last_mut() {
            Some((range, last)) if range.end == i && *last == value => range.end = i + 1,
            _ => runs.push((i..i + 1, value)),
        }
    }
    runs
}
