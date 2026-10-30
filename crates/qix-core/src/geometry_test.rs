use crate::{Dir, Point};

const ALL: [Dir; 4] = [Dir::Up, Dir::Down, Dir::Left, Dir::Right];

#[test]
fn opposite_pairs_the_directions() {
    assert_eq!(Dir::Up.opposite(), Dir::Down);
    assert_eq!(Dir::Down.opposite(), Dir::Up);
    assert_eq!(Dir::Left.opposite(), Dir::Right);
    assert_eq!(Dir::Right.opposite(), Dir::Left);
}

#[test]
fn opposite_twice_is_the_same_direction() {
    for dir in ALL {
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
    for dir in ALL {
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
