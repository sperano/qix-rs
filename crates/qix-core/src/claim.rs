//! Closing a stix: the flood fills that find the free areas holding the
//! Qix, and the claim of every other free cell.

use crate::bitset::BitSet;
use crate::board::{Board, Cell, DrawKind};
use crate::qix::Qix;
use crate::stix::Stix;

/// What closing a stix claimed.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ClaimResult {
    /// The cells the stix claimed.
    pub cells: usize,
    /// The board's claimed fraction after the claim, from 0 to 1.
    pub fraction: f32,
    /// How the claimed cells were drawn: [`Stix::claim_kind`].
    pub kind: DrawKind,
}

/// What closing a stix did.
#[derive(Clone, Debug, PartialEq)]
pub struct CloseResult {
    /// What the stix claimed, or `None` when it claimed nothing.
    pub claim: Option<ClaimResult>,
    /// For each Qix, in the order given, the free cells of the area
    /// holding it: the [`regions`] of their [`Qix::cell`]s. Two Qix share
    /// an area exactly when their sets are equal, so a close that leaves
    /// them apart shows here even when it claims nothing.
    pub regions: Vec<BitSet>,
}

impl Board {
    /// Closes `stix`: [`claim`]s every free cell outside the areas holding
    /// a Qix of `qix`, each found from its [`Qix::cell`]. With no Qix,
    /// every free cell is claimed.
    ///
    /// The border follows by itself, since [`Board::is_border`] reads the
    /// cells: the stix's points now lie between the claimed cells and the
    /// free areas kept.
    pub fn close_stix(&mut self, stix: &Stix, qix: &[Qix]) -> CloseResult {
        let seeds: Vec<_> = qix.iter().map(Qix::cell).collect();
        let regions = regions(self, stix, &seeds);
        let cells = claim(self, &regions, stix.claim_kind());
        CloseResult {
            claim: (cells > 0).then(|| ClaimResult {
                cells,
                fraction: self.claimed_fraction(),
                kind: stix.claim_kind(),
            }),
            regions,
        }
    }
}

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
    flood(board, &Walls::new(board, stix), seed)
}

/// The [`flood_free`] set of each of `seeds`, in order. A seed in the area
/// of an earlier one gets a copy of its set instead of a new fill, so the
/// whole runs in `O(W * H)` per distinct area plus a copy per seed.
pub fn regions(board: &Board, stix: &Stix, seeds: &[(u16, u16)]) -> Vec<BitSet> {
    let walls = Walls::new(board, stix);
    let mut sets: Vec<BitSet> = Vec::with_capacity(seeds.len());
    for &seed in seeds {
        let known = is_free(board, seed)
            .then(|| cell_index(board, seed))
            .and_then(|index| sets.iter().find(|set| set.contains(index)));
        let set = known.cloned().unwrap_or_else(|| flood(board, &walls, seed));
        sets.push(set);
    }
    sets
}

/// Claims, as `kind`, every free cell that none of `kept` holds, and
/// returns how many it claimed. With nothing kept, every free cell goes.
pub fn claim(board: &mut Board, kept: &[BitSet], kind: DrawKind) -> usize {
    let claimed = Cell::Claimed(kind);
    let mut count = 0;
    for y in 0..board.height() {
        for x in 0..board.width() {
            let index = cell_index(board, (x, y));
            if is_free(board, (x, y)) && !kept.iter().any(|set| set.contains(index)) {
                board.set_cell(x, y, claimed);
                count += 1;
            }
        }
    }
    count
}

/// The free cells reachable from `seed` without crossing `walls`: see
/// [`flood_free`].
fn flood(board: &Board, walls: &Walls, seed: (u16, u16)) -> BitSet {
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
