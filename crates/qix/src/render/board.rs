//! The board: claimed cells, the border, the stix and the marker.
//!
//! Positions are in playfield units (one unit per cell, lattice point
//! `(x, y)` at `(x, y)`) until [`paint`] maps them to the screen.

use std::ops::Range;

use eframe::egui::{Mesh, Painter, Pos2, Rect, Shape, Stroke, emath::RectTransform, pos2, vec2};
use qix_core::{Board, Cell, Dir, DrawKind, Game, Point, Stix};

use crate::palette;

/// Width of the border lines, in points.
const BORDER_WIDTH: f32 = 2.0;

/// Width of the stix, in points.
const STIX_WIDTH: f32 = 2.0;

/// Distance from the marker's centre to each of its diamond's corners, in
/// points.
const MARKER_RADIUS: f32 = 5.0;

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

/// Paints `game`'s board through `to_screen`: the claimed cells, then the
/// border, the stix and the marker on top.
pub fn paint(painter: &Painter, to_screen: RectTransform, game: &Game) {
    let board = game.board();
    painter.add(claimed_mesh(board, to_screen));
    let border = Stroke::new(BORDER_WIDTH, palette::BORDER);
    painter.extend(border_segments(board).into_iter().map(|(from, to)| {
        let ends = [screen(to_screen, from), screen(to_screen, to)];
        Shape::line_segment(square_capped(ends, BORDER_WIDTH), border)
    }));
    if let Some(stix) = game.stix() {
        paint_stix(painter, to_screen, stix);
    }
    let marker = screen(to_screen, game.marker().pos);
    painter.add(Shape::convex_polygon(
        diamond(marker, MARKER_RADIUS).to_vec(),
        palette::MARKER,
        Stroke::NONE,
    ));
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

/// The corners of a diamond centred on `center`, each `radius` from it,
/// clockwise from the top.
pub fn diamond(center: Pos2, radius: f32) -> [Pos2; 4] {
    [
        center - vec2(0.0, radius),
        center + vec2(radius, 0.0),
        center + vec2(0.0, radius),
        center - vec2(radius, 0.0),
    ]
}

/// The ends of a line `width` wide from `ends[0]` to `ends[1]`, each pushed
/// out by half the width. egui ends an open line flush with its end
/// points, so where two border lines meet at a corner the corner's outer
/// square would stay unpainted; the extension squares it off.
pub fn square_capped([from, to]: [Pos2; 2], width: f32) -> [Pos2; 2] {
    let extension = (to - from).normalized() * (width / 2.0);
    [from - extension, to + extension]
}

/// The claimed cells as one mesh of rectangles. A mesh is not
/// anti-aliased, so the rows meet without the faint seams separate filled
/// rectangles would leave between them.
fn claimed_mesh(board: &Board, to_screen: RectTransform) -> Mesh {
    let mut mesh = Mesh::default();
    for run in claimed_runs(board) {
        mesh.add_colored_rect(
            to_screen.transform_rect(run.rect()),
            palette::fill(run.kind),
        );
    }
    mesh
}

/// Paints the stix, each run of segments in the colour of its kind.
fn paint_stix(painter: &Painter, to_screen: RectTransform, stix: &Stix) {
    for (kind, points) in stix_runs(stix) {
        let points = points.iter().map(|&p| screen(to_screen, p)).collect();
        painter.line(points, Stroke::new(STIX_WIDTH, palette::stix(kind)));
    }
}

/// The screen position of lattice point `point`.
fn screen(to_screen: RectTransform, point: Point) -> Pos2 {
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
