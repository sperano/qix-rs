use crate::{Dir, Point, Vec2, in_triangle, segments_intersect};

#[test]
fn opposite_pairs_the_directions() {
    assert_eq!(Dir::Up.opposite(), Dir::Down);
    assert_eq!(Dir::Down.opposite(), Dir::Up);
    assert_eq!(Dir::Left.opposite(), Dir::Right);
    assert_eq!(Dir::Right.opposite(), Dir::Left);
}

#[test]
fn opposite_twice_is_the_same_direction() {
    for dir in Dir::ALL {
        assert_eq!(dir.opposite().opposite(), dir);
    }
}

#[test]
fn step_moves_one_lattice_unit_with_y_growing_down() {
    let point = Point::new(5, 7);
    assert_eq!(Dir::Up.step(point), Some(Point::new(5, 6)));
    assert_eq!(Dir::Down.step(point), Some(Point::new(5, 8)));
    assert_eq!(Dir::Left.step(point), Some(Point::new(4, 7)));
    assert_eq!(Dir::Right.step(point), Some(Point::new(6, 7)));
}

#[test]
fn stepping_back_returns_to_the_start() {
    let point = Point::new(3, 3);
    for dir in Dir::ALL {
        let next = dir.step(point).and_then(|next| dir.opposite().step(next));
        assert_eq!(next, Some(point), "{dir:?}");
    }
}

#[test]
fn step_past_the_coordinate_range_is_none() {
    assert_eq!(Dir::Up.step(Point::new(4, 0)), None);
    assert_eq!(Dir::Left.step(Point::new(0, 4)), None);
    assert_eq!(Dir::Down.step(Point::new(4, u16::MAX)), None);
    assert_eq!(Dir::Right.step(Point::new(u16::MAX, 4)), None);
}

#[test]
fn between_points_the_way_of_a_straight_run() {
    let from = Point::new(3, 3);
    let runs = [
        (Point::new(3, 1), Dir::Up),
        (Point::new(3, 4), Dir::Down),
        (Point::new(0, 3), Dir::Left),
        (Point::new(5, 3), Dir::Right),
    ];

    for (to, dir) in runs {
        assert_eq!(Dir::between(from, to), Some(dir), "{to:?}");
    }
}

#[test]
fn between_has_no_direction_for_the_same_point_or_a_diagonal() {
    let from = Point::new(3, 3);

    assert_eq!(Dir::between(from, from), None);
    assert_eq!(Dir::between(from, Point::new(4, 4)), None);
}

fn v(x: f32, y: f32) -> Vec2 {
    Vec2::new(x, y)
}

#[test]
fn crossing_segments_intersect() {
    assert!(segments_intersect(
        v(0.0, 0.0),
        v(2.0, 2.0),
        v(0.0, 2.0),
        v(2.0, 0.0)
    ));
    assert!(segments_intersect(
        v(1.0, 0.0),
        v(1.0, 4.0),
        v(0.0, 2.0),
        v(3.0, 2.0)
    ));
}

#[test]
fn segments_touching_at_an_end_intersect() {
    assert!(segments_intersect(
        v(0.0, 0.0),
        v(1.0, 1.0),
        v(1.0, 1.0),
        v(2.0, 0.0)
    ));
    assert!(segments_intersect(
        v(1.0, 0.0),
        v(1.0, 2.0),
        v(0.0, 2.0),
        v(3.0, 2.0)
    ));
}

#[test]
fn overlapping_collinear_segments_intersect() {
    assert!(segments_intersect(
        v(0.0, 1.0),
        v(3.0, 1.0),
        v(2.0, 1.0),
        v(5.0, 1.0)
    ));
}

#[test]
fn parallel_segments_do_not_intersect() {
    assert!(!segments_intersect(
        v(0.0, 0.0),
        v(3.0, 0.0),
        v(0.0, 1.0),
        v(3.0, 1.0)
    ));
}

#[test]
fn disjoint_collinear_segments_do_not_intersect() {
    assert!(!segments_intersect(
        v(0.0, 1.0),
        v(1.0, 1.0),
        v(2.0, 1.0),
        v(3.0, 1.0)
    ));
}

#[test]
fn segments_whose_lines_cross_elsewhere_do_not_intersect() {
    assert!(!segments_intersect(
        v(0.0, 0.0),
        v(1.0, 1.0),
        v(3.0, 0.0),
        v(2.0, 1.0)
    ));
    assert!(!segments_intersect(
        v(1.0, 0.0),
        v(1.0, 1.5),
        v(0.0, 2.0),
        v(3.0, 2.0)
    ));
}

#[test]
fn intersecting_is_symmetric() {
    let (a, b, c, d) = (v(0.0, 0.0), v(2.0, 2.0), v(0.0, 2.0), v(2.0, 0.0));
    assert!(segments_intersect(c, d, a, b));
    assert!(segments_intersect(b, a, d, c));
}

/// A right triangle with its right angle at the origin.
fn right_triangle() -> [Vec2; 3] {
    [
        Vec2::new(0.0, 0.0),
        Vec2::new(4.0, 0.0),
        Vec2::new(0.0, 4.0),
    ]
}

#[test]
fn a_point_inside_a_triangle_is_in_it_either_way_round() {
    let triangle = right_triangle();
    let [p, q, r] = triangle;
    let point = Vec2::new(1.0, 1.0);

    assert!(in_triangle(point, triangle));
    assert!(in_triangle(point, [p, r, q]));
}

#[test]
fn a_point_on_a_side_or_a_corner_is_in_the_triangle() {
    let triangle = right_triangle();

    assert!(in_triangle(Vec2::new(2.0, 0.0), triangle));
    assert!(in_triangle(Vec2::new(2.0, 2.0), triangle));
    assert!(in_triangle(Vec2::new(0.0, 4.0), triangle));
}

#[test]
fn a_point_outside_a_triangle_is_not_in_it() {
    let triangle = right_triangle();

    assert!(!in_triangle(Vec2::new(3.0, 3.0), triangle));
    assert!(!in_triangle(Vec2::new(-0.1, 1.0), triangle));
}

#[test]
fn a_triangle_with_no_area_holds_no_point() {
    let flat = [
        Vec2::new(0.0, 0.0),
        Vec2::new(2.0, 0.0),
        Vec2::new(4.0, 0.0),
    ];

    assert!(!in_triangle(Vec2::new(1.0, 0.0), flat));
    assert!(!in_triangle(Vec2::new(6.0, 0.0), flat));
}
