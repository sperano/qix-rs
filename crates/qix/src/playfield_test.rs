use eframe::egui::{Pos2, Rect, Vec2, pos2, vec2};
use qix_core::tuning::{PLAYFIELD_HEIGHT, PLAYFIELD_WIDTH};

use crate::playfield::{letterbox, playfield_units, to_screen};

/// Tolerance for comparing screen coordinates, in points.
const EPSILON: f32 = 1e-3;

/// Window areas to fit the playfield into: wide, tall, exact and tiny.
const AREAS: [Vec2; 5] = [
    vec2(1920.0, 1080.0),
    vec2(600.0, 1000.0),
    vec2(800.0, 600.0),
    vec2(333.0, 250.0),
    vec2(1.0, 1.0),
];

fn aspect() -> f32 {
    f32::from(PLAYFIELD_WIDTH) / f32::from(PLAYFIELD_HEIGHT)
}

fn assert_near(actual: Pos2, expected: Pos2) {
    assert!(
        (actual - expected).length() < EPSILON,
        "{actual:?} != {expected:?}"
    );
}

fn area(size: Vec2) -> Rect {
    Rect::from_min_size(pos2(10.0, 20.0), size)
}

#[test]
fn the_playfield_keeps_its_aspect_ratio_at_any_size() {
    for size in AREAS {
        let field = letterbox(area(size));
        let ratio = field.width() / field.height();
        assert!((ratio - aspect()).abs() < EPSILON, "{size:?}: {ratio}");
    }
}

#[test]
fn the_playfield_fills_one_axis_and_stays_centred_inside() {
    for size in AREAS {
        let available = area(size);
        let field = letterbox(available);
        let fills_width = (field.width() - available.width()).abs() < EPSILON;
        let fills_height = (field.height() - available.height()).abs() < EPSILON;
        assert!(fills_width || fills_height, "{size:?}: {field:?}");
        assert!(available.expand(EPSILON).contains_rect(field), "{size:?}");
        assert_near(field.center(), available.center());
    }
}

#[test]
fn a_wide_window_letterboxes_left_and_right() {
    let field = letterbox(Rect::from_min_size(Pos2::ZERO, vec2(1600.0, 600.0)));
    assert_near(field.min, pos2(400.0, 0.0));
    assert_near(field.max, pos2(1200.0, 600.0));
}

#[test]
fn an_empty_area_gives_an_empty_playfield() {
    let field = letterbox(Rect::from_min_size(Pos2::ZERO, Vec2::ZERO));
    assert_eq!(field.size(), Vec2::ZERO);
}

#[test]
fn playfield_corners_map_to_the_letterbox_corners() {
    let available = area(vec2(1000.0, 500.0));
    let transform = to_screen(available);
    let units = playfield_units();
    let field = letterbox(available);
    assert_near(transform.transform_pos(units.min), field.min);
    assert_near(transform.transform_pos(units.max), field.max);
}
