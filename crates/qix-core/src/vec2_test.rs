use std::f32::consts::FRAC_PI_2;

use crate::{Point, Vec2};

/// Rounding slack of the trigonometric results.
const EPSILON: f32 = 1e-6;

fn assert_near(actual: Vec2, expected: Vec2) {
    let error = (actual - expected).length();
    assert!(error < EPSILON, "{actual:?} != {expected:?}");
}

#[test]
fn adds_subtracts_scales_and_negates() {
    let (a, b) = (Vec2::new(1.0, 2.0), Vec2::new(3.0, -1.0));

    assert_eq!(a + b, Vec2::new(4.0, 1.0));
    assert_eq!(a - b, Vec2::new(-2.0, 3.0));
    assert_eq!(a * 2.0, Vec2::new(2.0, 4.0));
    assert_eq!(-a, Vec2::new(-1.0, -2.0));
}

#[test]
fn measures_its_length() {
    assert_eq!(Vec2::new(3.0, -4.0).length(), 5.0);
}

#[test]
fn scales_to_a_length() {
    assert_eq!(Vec2::new(3.0, -4.0).with_length(10.0), Vec2::new(6.0, -8.0));
    assert_eq!(Vec2::default().with_length(10.0), Vec2::default());
}

#[test]
fn finds_the_midpoint() {
    let midpoint = Vec2::new(1.0, 2.0).midpoint(Vec2::new(3.0, 6.0));

    assert_eq!(midpoint, Vec2::new(2.0, 4.0));
}

#[test]
fn a_quarter_turn_takes_x_onto_y() {
    assert_near(Vec2::new(2.0, 0.0).rotated(FRAC_PI_2), Vec2::new(0.0, 2.0));
}

#[test]
fn from_angle_is_the_turned_unit_vector() {
    assert_near(Vec2::from_angle(FRAC_PI_2), Vec2::new(0.0, 1.0));
    assert_near(Vec2::from_angle(0.0), Vec2::new(1.0, 0.0));
}

#[test]
fn cross_is_signed_by_the_turn() {
    let (x, y) = (Vec2::new(1.0, 0.0), Vec2::new(0.0, 1.0));
    assert_eq!(x.cross(y), 1.0);
    assert_eq!(y.cross(x), -1.0);
    assert_eq!(x.cross(x * 3.0), 0.0);
}

#[test]
fn a_lattice_point_is_the_corner_of_its_cell() {
    assert_eq!(Vec2::from(Point::new(3, 7)), Vec2::new(3.0, 7.0));
}
