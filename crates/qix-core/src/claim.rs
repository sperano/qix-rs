//! Closing a stix: the flood fill that finds the free area holding a seed
//! cell, and the claim of every other free cell.

use crate::bitset::BitSet;
use crate::board::{Board, Cell};
use crate::stix::Stix;

/// The free cells reachable from the cell `seed` by steps between
/// 4-adjacent free cells that do not cross a segment of `stix`.
///
/// Bit `y * width + x` of the result stands for cell `(x, y)`. The step
/// between cells `(x, y)` and `(x + 1, y)` crosses the lattice edge from
/// point `(x + 1, y)` to `(x + 1, y + 1)`; the step between `(x, y)` and
/// `(x, y + 1)` the edge from `(x, y + 1)` to `(x + 1, y + 1)`. A seed off
/// the board or on a claimed cell reaches nothing. Runs in `O(W * H)`
/// with an explicit stack.
pub fn flood_free(board: &Board, stix: &Stix, seed: (u16, u16)) -> BitSet {
    let walls = Walls::new(board, stix);
    let mut reached = BitSet::new(usize::from(board.width()) * usize::from(board.height()));
    if !is_free(board, seed) {
        return reached;
    }
    reached.insert(cell_index(board, seed));
    let mut stack = vec![seed];
    while let Some(cell) = stack.pop() {
        for next in walls.open_neighbours(cell) {
            if is_free(board, next) && reached.insert(cell_index(board, next)) {
                stack.push(next);
            }
        }
    }
    reached
}

/// Claims, as [`Stix::claim_kind`], every free cell that [`flood_free`]
/// does not reach from `seed`, and returns how many it claimed.
///
/// A seed that reaches nothing (off the board or claimed) claims every
/// free cell: the caller picks a free seed in the area to keep.
pub fn claim(board: &mut Board, stix: &Stix, seed: (u16, u16)) -> usize {
    let kept = flood_free(board, stix, seed);
    let claimed = Cell::Claimed(stix.claim_kind());
    let mut count = 0;
    for y in 0..board.height() {
        for x in 0..board.width() {
            if is_free(board, (x, y)) && !kept.contains(cell_index(board, (x, y))) {
                board.set_cell(x, y, claimed);
                count += 1;
            }
        }
    }
    count
}

/// The unit lattice edges a stix covers, which a flood fill may not cross.
struct Walls {
    width: u16,
    height: u16,
    /// Edge from point `(x, y)` to `(x, y + 1)` at bit `y * (width + 1) + x`.
    vertical: BitSet,
    /// Edge from point `(x, y)` to `(x + 1, y)` at bit `y * width + x`.
    horizontal: BitSet,
}

impl Walls {
    /// The edges `stix` covers on `board`'s lattice.
    fn new(board: &Board, stix: &Stix) -> Self {
        let (width, height) = (board.width(), board.height());
        let (w, h) = (usize::from(width), usize::from(height));
        let mut walls = Self {
            width,
            height,
            vertical: BitSet::new((w + 1) * h),
            horizontal: BitSet::new(w * (h + 1)),
        };
        for pair in stix.points().windows(2) {
            let (a, b) = (pair[0], pair[1]);
            if a.x == b.x {
                (a.y.min(b.y)..a.y.max(b.y)).for_each(|y| walls.add_vertical(a.x, y));
            } else {
                (a.x.min(b.x)..a.x.max(b.x)).for_each(|x| walls.add_horizontal(x, a.y));
            }
        }
        walls
    }

    /// Covers the edge from point `(x, y)` down to `(x, y + 1)`, unless it
    /// is off the lattice.
    fn add_vertical(&mut self, x: u16, y: u16) {
        if let Some(index) = self.vertical_index(x, y) {
            self.vertical.insert(index);
        }
    }

    /// Covers the edge from point `(x, y)` right to `(x + 1, y)`, unless
    /// it is off the lattice.
    fn add_horizontal(&mut self, x: u16, y: u16) {
        if let Some(index) = self.horizontal_index(x, y) {
            self.horizontal.insert(index);
        }
    }

    /// The edge from point `(x, y)` down to `(x, y + 1)` is covered.
    fn has_vertical(&self, x: u16, y: u16) -> bool {
        self.vertical_index(x, y)
            .is_some_and(|index| self.vertical.contains(index))
    }

    /// The edge from point `(x, y)` right to `(x + 1, y)` is covered.
    fn has_horizontal(&self, x: u16, y: u16) -> bool {
        self.horizontal_index(x, y)
            .is_some_and(|index| self.horizontal.contains(index))
    }

    /// The bit of the vertical edge below point `(x, y)`, if on the lattice.
    fn vertical_index(&self, x: u16, y: u16) -> Option<usize> {
        let w = usize::from(self.width);
        (x <= self.width && y < self.height).then(|| usize::from(y) * (w + 1) + usize::from(x))
    }

    /// The bit of the horizontal edge right of point `(x, y)`, if on the
    /// lattice.
    fn horizontal_index(&self, x: u16, y: u16) -> Option<usize> {
        let w = usize::from(self.width);
        (x < self.width && y <= self.height).then(|| usize::from(y) * w + usize::from(x))
    }

    /// The cells on the board 4-adjacent to `cell` whose shared edge with
    /// it no wall covers.
    fn open_neighbours(&self, (x, y): (u16, u16)) -> impl Iterator<Item = (u16, u16)> {
        let left = (x > 0 && !self.has_vertical(x, y)).then(|| (x - 1, y));
        let right = (x + 1 < self.width && !self.has_vertical(x + 1, y)).then(|| (x + 1, y));
        let up = (y > 0 && !self.has_horizontal(x, y)).then(|| (x, y - 1));
        let down = (y + 1 < self.height && !self.has_horizontal(x, y + 1)).then(|| (x, y + 1));
        [left, right, up, down].into_iter().flatten()
    }
}

/// Cell `(x, y)` is on the board and free.
fn is_free(board: &Board, (x, y): (u16, u16)) -> bool {
    board.cell(i32::from(x), i32::from(y)) == Cell::Free
}

/// The bit of cell `(x, y)` in a [`flood_free`] result; the cell must be
/// on the board.
fn cell_index(board: &Board, (x, y): (u16, u16)) -> usize {
    usize::from(y) * usize::from(board.width()) + usize::from(x)
}
