//! The Sparx: enemies walking the border graph, and Super ones the stix,
//! that kill the marker on contact.

use crate::board::Board;
use crate::geometry::{Dir, Point};
use crate::rng::Rng;
use crate::stix::Stix;

/// What a Sparx is allowed to walk on.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SparxKind {
    /// Walks the border only.
    Normal,
    /// Also chases the marker along the stix: from the stix's start it
    /// walks the stix towards the marker.
    Super,
}

/// A Sparx on a lattice point of the border, or of the stix while a
/// [`SparxKind::Super`] one chases along it, heading `dir`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Sparx {
    pub pos: Point,
    pub dir: Dir,
    pub kind: SparxKind,
}

impl Sparx {
    /// The two [`SparxKind::Normal`] Sparx of a level start: on the border
    /// point nearest the centre of `board`'s top edge, one heading left
    /// and the other right.
    pub fn spawn_pair(board: &Board) -> [Self; 2] {
        let pos = top_centre(board);
        [Dir::Left, Dir::Right].map(|dir| Self {
            pos,
            dir,
            kind: SparxKind::Normal,
        })
    }

    /// `count` [`SparxKind::Normal`] Sparx on the border point nearest the
    /// centre of `board`'s top edge, like [`Sparx::spawn_pair`]: heading
    /// left and right in turn, the first left.
    pub fn spawn_at_top(board: &Board, count: usize) -> impl Iterator<Item = Self> {
        let pos = top_centre(board);
        [Dir::Left, Dir::Right]
            .into_iter()
            .cycle()
            .take(count)
            .map(move |dir| Self {
                pos,
                dir,
                kind: SparxKind::Normal,
            })
    }

    /// Moves one step along a border edge of `board`: straight on when it
    /// can, else a random sideways edge, back only at a dead end; a Sparx
    /// with no border edge stays put.
    pub fn step(&mut self, board: &Board, rng: &mut Rng) {
        let edges = || board.border_neighbours(self.pos).map(|(dir, _)| dir);
        let Some(dir) = turn(self.dir, edges, rng) else {
            return;
        };
        if let Some(next) = dir.step(self.pos) {
            self.pos = next;
            self.dir = dir;
        }
    }

    /// The Sparx walks `stix` instead of the border: it is
    /// [`SparxKind::Super`] and on a point of the stix, its start included.
    pub fn chases_along(&self, stix: &Stix) -> bool {
        self.kind == SparxKind::Super && stix.contains_point(self.pos)
    }

    /// Moves one lattice step along `stix` towards its head, where the
    /// marker is; a Sparx at the head or off the stix stays put.
    pub fn step_along_stix(&mut self, stix: &Stix) {
        let Some(dir) = stix.towards_head(self.pos) else {
            return;
        };
        if let Some(next) = dir.step(self.pos) {
            self.pos = next;
            self.dir = dir;
        }
    }

    /// Moves to the border point of `board` nearest the Sparx's point (the
    /// fewest lattice steps away, the first in reading order among equally
    /// near ones) when that point is no longer on the border; its heading
    /// is kept.
    pub fn snap_to_border(&mut self, board: &Board) {
        if board.is_border(self.pos) {
            return;
        }
        if let Some(nearest) = nearest_border_point(board, self.pos) {
            self.pos = nearest;
        }
    }

    /// The Sparx, having moved from `from` to its point this tick, touched
    /// the marker, which moved from `marker_from` to `marker_to`: they end
    /// on the same point, or they swapped points.
    pub fn hits_marker(&self, from: Point, marker_from: Point, marker_to: Point) -> bool {
        self.pos == marker_to || (self.pos == marker_from && from == marker_to)
    }
}

/// The direction a Sparx heading `dir` leaves a point by, given the
/// directions of the point's border edges as `edges` yields them: straight
/// on when it can; else one of the edges other than back, drawn from `rng`
/// when there are several; back only at a dead end. `None` when the point
/// has no edge at all.
///
/// `rng` is drawn from only when there is a choice to make.
pub(crate) fn turn<I>(dir: Dir, edges: impl Fn() -> I, rng: &mut Rng) -> Option<Dir>
where
    I: Iterator<Item = Dir>,
{
    if edges().any(|edge| edge == dir) {
        return Some(dir);
    }
    let back = dir.opposite();
    let sideways = || edges().filter(move |&edge| edge != back);
    let choice = match sideways().count() {
        0 => return edges().next(),
        1 => 0,
        choices => rng.below(choices),
    };
    sideways().nth(choice)
}

/// The border point of `board` nearest the centre of its top edge, or
/// that centre when the board has no border.
fn top_centre(board: &Board) -> Point {
    let centre = Point::new(board.width() / 2, 0);
    nearest_border_point(board, centre).unwrap_or(centre)
}

/// The border point of `board` the fewest lattice steps (the Manhattan
/// distance) from `point`, the first in reading order among equally near
/// ones; `None` when the board has no border.
pub(crate) fn nearest_border_point(board: &Board, point: Point) -> Option<Point> {
    let (w, h) = (board.width(), board.height());
    (0..=h)
        .flat_map(|y| (0..=w).map(move |x| Point::new(x, y)))
        .filter(|&candidate| board.is_border(candidate))
        .min_by_key(|candidate| {
            u32::from(point.x.abs_diff(candidate.x)) + u32::from(point.y.abs_diff(candidate.y))
        })
}
