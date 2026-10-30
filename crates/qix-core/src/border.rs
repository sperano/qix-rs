//! The border graph: the lattice points and edges where free cells meet
//! claimed ones. Sparx walk it and the marker follows it.

use crate::board::{Board, Cell};
use crate::geometry::{Dir, Point};

impl Board {
    /// The lattice point `point` is on the border: its four surrounding
    /// cells, the outside counting as claimed, hold both free and claimed
    /// cells. A point off the board's lattice never is.
    pub fn is_border(&self, point: Point) -> bool {
        let cells = self.cells_around(point);
        let claimed = cells.iter().filter(|cell| cell.is_claimed()).count();
        claimed > 0 && claimed < cells.len()
    }

    /// The lattice points one step from `point` along a border edge, with
    /// the direction of each step, in the order of [`Dir::ALL`].
    pub fn border_neighbours(&self, point: Point) -> impl Iterator<Item = (Dir, Point)> + '_ {
        let cells = self.cells_around(point);
        Dir::ALL.into_iter().filter_map(move |dir| {
            let next = dir.step(point)?;
            is_border_edge(cells, dir).then_some((dir, next))
        })
    }

    /// The edge from `point` one step towards `dir` runs through free
    /// space: the cells on both its sides are free. An edge leaving the
    /// board's lattice never does, since the outside counts as claimed.
    pub fn is_free_edge(&self, point: Point, dir: Dir) -> bool {
        let (side, other_side) = edge_sides(self.cells_around(point), dir);
        side == Cell::Free && other_side == Cell::Free
    }
}

/// The edge from a lattice point one step towards `dir` is border, given
/// the point's [`Board::cells_around`]: the two cells on the edge's sides
/// differ in claimed-ness.
fn is_border_edge(cells: [Cell; 4], dir: Dir) -> bool {
    let (side, other_side) = edge_sides(cells, dir);
    side.is_claimed() != other_side.is_claimed()
}

/// The two cells on the sides of the edge from a lattice point one step
/// towards `dir`, given the point's [`Board::cells_around`].
fn edge_sides(cells: [Cell; 4], dir: Dir) -> (Cell, Cell) {
    let [top_left, top_right, bottom_left, bottom_right] = cells;
    match dir {
        Dir::Up => (top_left, top_right),
        Dir::Down => (bottom_left, bottom_right),
        Dir::Left => (top_left, bottom_left),
        Dir::Right => (top_right, bottom_right),
    }
}
