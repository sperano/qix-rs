use eframe::egui::{Color32, Mesh, Pos2, Rect, pos2};
use qix_core::SparxKind;

use crate::palette;
use crate::render::sprite::{
    BODY, CORE, HIGHLIGHT, INNER, MARKER, Mask, OUTLINE, PIXEL, SPARK_FRAMES, TRANSPARENT, marker,
    mask_size, mesh, spark,
};

/// Tolerance for comparing screen coordinates, in points.
const EPSILON: f32 = 1e-3;

/// A centre to paint sprites around.
const CENTRE: Pos2 = pos2(40.0, 30.0);

/// Points per sprite pixel in the tests that do not use [`PIXEL`].
const BIG_PIXEL: f32 = 4.0;

const WHITE: Color32 = Color32::WHITE;

fn masks() -> Vec<Mask> {
    let mut masks = vec![MARKER];
    masks.extend(SPARK_FRAMES);
    masks
}

/// The squares of `mesh`, in painting order, with their colours.
fn squares(mesh: &Mesh) -> Vec<(Rect, Color32)> {
    mesh.indices
        .chunks(6)
        .map(|quad| {
            let vertices: Vec<_> = quad.iter().map(|&i| mesh.vertices[i as usize]).collect();
            let points: Vec<_> = vertices.iter().map(|v| v.pos).collect();
            (Rect::from_points(&points), vertices[0].color)
        })
        .collect()
}

fn bounds(mesh: &Mesh) -> Rect {
    squares(mesh)
        .into_iter()
        .map(|(rect, _)| rect)
        .reduce(Rect::union)
        .unwrap()
}

fn count(mesh: &Mesh, colour: Color32) -> usize {
    squares(mesh).iter().filter(|(_, c)| *c == colour).count()
}

fn roles(mask: Mask, role: char) -> usize {
    mask.iter()
        .flat_map(|row| row.chars())
        .filter(|&c| c == role)
        .count()
}

#[test]
fn every_row_of_a_mask_has_the_same_odd_width_and_there_is_an_odd_number_of_rows() {
    for mask in masks() {
        let (width, height) = mask_size(mask);
        assert!(width % 2 == 1 && height % 2 == 1, "{mask:?}");
        assert!(
            mask.iter().all(|row| row.chars().count() == width),
            "{mask:?}"
        );
    }
}

#[test]
fn the_size_of_an_empty_mask_is_zero() {
    assert_eq!(mask_size(&[]), (0, 0));
}

#[test]
fn the_middle_pixel_of_every_mask_is_painted() {
    for mask in masks() {
        let (width, height) = mask_size(mask);
        let middle = mask[height / 2].chars().nth(width / 2).unwrap();
        assert_ne!(middle, TRANSPARENT, "{mask:?}");
    }
}

#[test]
fn the_marker_is_a_diamond_symmetric_about_its_middle() {
    for (row, mirror) in MARKER.iter().zip(MARKER.iter().rev()) {
        let outline = |row: &str| row.chars().map(|c| c == TRANSPARENT).collect::<Vec<_>>();
        assert_eq!(outline(row), outline(mirror));
        let reversed: Vec<_> = outline(row).into_iter().rev().collect();
        assert_eq!(outline(row), reversed);
    }
}

#[test]
fn the_spark_frames_share_a_core_and_differ_in_their_arms() {
    let [plus, cross] = SPARK_FRAMES;
    assert_ne!(plus, cross);
    assert_eq!(roles(plus, CORE), roles(cross, CORE));
    assert!(roles(plus, BODY) > 0 && roles(cross, BODY) > 0);
}

#[test]
fn a_mesh_has_one_square_per_painted_pixel() {
    let mask: Mask = &["..X", ".XX", "X.X"];
    let mesh = mesh(mask, CENTRE, BIG_PIXEL, |_| Some(WHITE));
    assert_eq!(squares(&mesh).len(), 5);
}

#[test]
fn transparent_pixels_stay_unpainted_whatever_the_caller_maps_them_to() {
    let mask: Mask = &["...", ".X.", "..."];
    let mesh = mesh(mask, CENTRE, BIG_PIXEL, |_| Some(WHITE));
    assert_eq!(squares(&mesh).len(), 1);
}

#[test]
fn roles_without_a_colour_stay_unpainted() {
    let mask: Mask = &["XYX"];
    let only_x = |role| (role == 'X').then_some(WHITE);
    assert_eq!(squares(&mesh(mask, CENTRE, BIG_PIXEL, only_x)).len(), 2);
}

#[test]
fn a_mesh_is_centred_on_its_point_and_sized_by_the_pixel() {
    let mask: Mask = &["XXX", "XXX", "XXX", "XXX", "XXX"];
    let rect = bounds(&mesh(mask, CENTRE, BIG_PIXEL, |_| Some(WHITE)));
    assert!(rect.center().distance(CENTRE) < EPSILON, "{rect:?}");
    assert!((rect.width() - 3.0 * BIG_PIXEL).abs() < EPSILON, "{rect:?}");
    assert!(
        (rect.height() - 5.0 * BIG_PIXEL).abs() < EPSILON,
        "{rect:?}"
    );
}

#[test]
fn the_squares_are_axis_aligned_and_a_pixel_wide() {
    let mask: Mask = &["XX", "XX"];
    for (rect, _) in squares(&mesh(mask, CENTRE, BIG_PIXEL, |_| Some(WHITE))) {
        assert!((rect.width() - BIG_PIXEL).abs() < EPSILON, "{rect:?}");
        assert!((rect.height() - BIG_PIXEL).abs() < EPSILON, "{rect:?}");
    }
}

#[test]
fn pixels_keep_their_place_in_the_mask() {
    let mask: Mask = &["X.", ".Y"];
    let colour_of = |role| {
        Some(if role == 'X' {
            Color32::RED
        } else {
            Color32::BLUE
        })
    };
    let squares = squares(&mesh(mask, CENTRE, BIG_PIXEL, colour_of));
    let (top_left, red) = squares[0];
    let (bottom_right, blue) = squares[1];
    assert_eq!((red, blue), (Color32::RED, Color32::BLUE));
    assert!(top_left.max.x <= bottom_right.min.x + EPSILON);
    assert!(top_left.max.y <= bottom_right.min.y + EPSILON);
}

#[test]
fn the_marker_is_painted_in_its_outline_highlight_and_inner_colours() {
    let inner = Color32::from_rgb(1, 2, 3);
    let mesh = marker(CENTRE, PIXEL, inner);
    assert_eq!(
        count(&mesh, palette::MARKER_OUTLINE),
        roles(MARKER, OUTLINE)
    );
    assert_eq!(
        count(&mesh, palette::MARKER_HIGHLIGHT),
        roles(MARKER, HIGHLIGHT)
    );
    assert_eq!(count(&mesh, inner), roles(MARKER, INNER));
    let painted = roles(MARKER, OUTLINE) + roles(MARKER, HIGHLIGHT) + roles(MARKER, INNER);
    assert_eq!(squares(&mesh).len(), painted);
}

#[test]
fn the_marker_is_centred_on_its_point() {
    let rect = bounds(&marker(CENTRE, PIXEL, palette::MARKER));
    assert!(rect.center().distance(CENTRE) < EPSILON, "{rect:?}");
    let (width, height) = mask_size(MARKER);
    assert!(
        (rect.width() - width as f32 * PIXEL).abs() < EPSILON,
        "{rect:?}"
    );
    assert!(
        (rect.height() - height as f32 * PIXEL).abs() < EPSILON,
        "{rect:?}"
    );
}

#[test]
fn a_spark_is_painted_in_the_body_and_core_colours_of_its_kind() {
    let cases = [
        (SparxKind::Normal, palette::SPARX, palette::SPARX_CORE),
        (
            SparxKind::Super,
            palette::SUPER_SPARX,
            palette::SUPER_SPARX_CORE,
        ),
    ];
    for (kind, body, core) in cases {
        for frame in SPARK_FRAMES {
            let mesh = spark(frame, CENTRE, kind);
            assert_eq!(count(&mesh, body), roles(frame, BODY));
            assert_eq!(count(&mesh, core), roles(frame, CORE));
            assert_eq!(
                squares(&mesh).len(),
                roles(frame, BODY) + roles(frame, CORE)
            );
        }
    }
}

#[test]
fn a_spark_is_centred_on_its_point_in_every_frame() {
    for frame in SPARK_FRAMES {
        let rect = bounds(&spark(frame, CENTRE, SparxKind::Normal));
        assert!(rect.center().distance(CENTRE) < EPSILON, "{rect:?}");
    }
}
