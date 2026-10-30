//! The board: a grid of cells, each free or claimed.

use crate::geometry::Point;

/// How a claimed area was drawn, which decides its colour and its points.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DrawKind {
    Fast,
    Slow,
}

/// One cell of the board.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Cell {
    Free,
    Claimed(DrawKind),
}

impl Cell {
    /// The cell is claimed, however it was drawn.
    pub const fn is_claimed(self) -> bool {
        matches!(self, Self::Claimed(_))
    }
}

/// What every cell outside the board reads as: claimed, so the board's
/// edges are border like the edge of any claimed area. The draw kind has
/// no meaning there; the outside is never painted or scored.
pub const OUTSIDE: Cell = Cell::Claimed(DrawKind::Fast);

/// A `width x height` grid of cells, row by row from the top left.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Board {
    width: u16,
    height: u16,
    cells: Vec<Cell>,
}

impl Board {
    /// A board of `width x height` free cells.
    ///
    /// # Panics
    ///
    /// If either dimension is zero.
    pub fn new(width: u16, height: u16) -> Self {
        assert!(width > 0 && height > 0, "empty board {width}x{height}");
        let len = usize::from(width) * usize::from(height);
        Self {
            width,
            height,
            cells: vec![Cell::Free; len],
        }
    }

    /// The width in cells.
    pub fn width(&self) -> u16 {
        self.width
    }

    /// The height in cells.
    pub fn height(&self) -> u16 {
        self.height
    }

    /// The cell at `(x, y)`, or [`OUTSIDE`] when that is off the board.
    pub fn cell(&self, x: i32, y: i32) -> Cell {
        self.index(x, y).map_or(OUTSIDE, |i| self.cells[i])
    }

    /// Sets the cell at `(x, y)`.
    ///
    /// # Panics
    ///
    /// If `(x, y)` is off the board.
    pub fn set_cell(&mut self, x: u16, y: u16, cell: Cell) {
        let Some(i) = self.index(i32::from(x), i32::from(y)) else {
            panic!(
                "cell ({x}, {y}) is off the {}x{} board",
                self.width, self.height
            );
        };
        self.cells[i] = cell;
    }

    /// The four cells touching the lattice point `point`, in reading
    /// order: top left, top right, bottom left, bottom right. Cells off
    /// the board read as [`OUTSIDE`].
    pub fn cells_around(&self, point: Point) -> [Cell; 4] {
        let (x, y) = (i32::from(point.x), i32::from(point.y));
        [
            self.cell(x - 1, y - 1),
            self.cell(x, y - 1),
            self.cell(x - 1, y),
            self.cell(x, y),
        ]
    }

    /// The share of the board's cells that are claimed, from 0 to 1.
    pub fn claimed_fraction(&self) -> f32 {
        let claimed = self.cells.iter().filter(|cell| cell.is_claimed()).count();
        claimed as f32 / self.cells.len() as f32
    }

    /// The position of cell `(x, y)` in `cells`, if it is on the board.
    fn index(&self, x: i32, y: i32) -> Option<usize> {
        let x = u16::try_from(x).ok().filter(|&x| x < self.width)?;
        let y = u16::try_from(y).ok().filter(|&y| y < self.height)?;
        Some(usize::from(y) * usize::from(self.width) + usize::from(x))
    }
}
