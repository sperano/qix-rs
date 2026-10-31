use crate::Vec2;
use crate::segment::cells;

fn walk(from: (f32, f32), to: (f32, f32)) -> Vec<(i32, i32)> {
    cells(Vec2::new(from.0, from.1), Vec2::new(to.0, to.1)).collect()
}

#[test]
fn a_segment_inside_one_cell_is_that_cell() {
    assert_eq!(walk((2.2, 3.1), (2.8, 3.9)), [(2, 3)]);
}

#[test]
fn a_point_is_its_cell() {
    assert_eq!(walk((2.5, 3.5), (2.5, 3.5)), [(2, 3)]);
}

#[test]
fn a_horizontal_segment_crosses_its_row() {
    assert_eq!(
        walk((0.5, 1.5), (3.5, 1.5)),
        [(0, 1), (1, 1), (2, 1), (3, 1)]
    );
}

#[test]
fn a_vertical_segment_walks_upwards() {
    assert_eq!(walk((1.5, 2.5), (1.5, 0.5)), [(1, 2), (1, 1), (1, 0)]);
}

#[test]
fn a_shallow_segment_steps_down_where_it_crosses_the_row() {
    let cells = walk((0.5, 0.2), (3.5, 1.2));

    assert_eq!(cells, [(0, 0), (1, 0), (2, 0), (2, 1), (3, 1)]);
}

#[test]
fn walking_backwards_visits_the_same_cells_in_reverse() {
    let (from, to) = ((0.3, 0.7), (5.9, 3.1));
    let mut backwards = walk(to, from);
    backwards.reverse();

    assert_eq!(walk(from, to), backwards);
}

#[test]
fn every_step_is_to_a_side_neighbour_and_the_walk_ends_at_the_end_cell() {
    let cells = walk((10.1, 4.9), (-3.7, -2.2));

    for pair in cells.windows(2) {
        let [(x0, y0), (x1, y1)] = pair else {
            unreachable!()
        };
        assert_eq!(x0.abs_diff(*x1) + y0.abs_diff(*y1), 1, "{pair:?}");
    }
    assert_eq!(cells.first(), Some(&(10, 4)));
    assert_eq!(cells.last(), Some(&(-4, -3)));
}

#[test]
fn a_segment_starting_on_a_boundary_moving_back_leaves_at_once() {
    assert_eq!(walk((2.0, 0.5), (1.5, 0.5)), [(2, 0), (1, 0)]);
}
