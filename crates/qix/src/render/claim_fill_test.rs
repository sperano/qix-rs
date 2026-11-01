use eframe::egui::{Context, RawInput, Rect, Shape, emath::RectTransform, pos2};
use qix_core::DrawKind;

use crate::palette;
use crate::render::board::CellRun;
use crate::render::claim_fill::{fill_colour, paint};

#[test]
fn the_animation_starts_on_the_flash() {
    for kind in [DrawKind::Fast, DrawKind::Slow] {
        assert_eq!(fill_colour(kind, 0.0), palette::CLAIM_FLASH);
    }
}

#[test]
fn the_animation_ends_on_the_fill_of_the_kind() {
    for kind in [DrawKind::Fast, DrawKind::Slow] {
        assert_eq!(fill_colour(kind, 1.0), palette::fill(kind));
    }
}

#[test]
fn progress_outside_the_animation_is_clamped() {
    let kind = DrawKind::Slow;
    assert_eq!(fill_colour(kind, -1.0), fill_colour(kind, 0.0));
    assert_eq!(fill_colour(kind, 2.0), fill_colour(kind, 1.0));
}

#[test]
fn the_colour_moves_from_the_flash_towards_the_fill() {
    let kind = DrawKind::Fast;
    let distance = |colour: eframe::egui::Color32| {
        let [r, g, b, _] = colour.to_array();
        let [fr, fg, fb, _] = palette::fill(kind).to_array();
        [r.abs_diff(fr), g.abs_diff(fg), b.abs_diff(fb)]
            .map(u32::from)
            .iter()
            .sum::<u32>()
    };
    let steps: Vec<_> = (0..=4)
        .map(|step| distance(fill_colour(kind, step as f32 / 4.0)))
        .collect();
    assert!(steps.is_sorted_by(|a, b| a > b), "{steps:?}");
}

#[test]
fn each_run_is_painted_in_place_in_the_colour_of_its_progress() {
    let runs = [
        CellRun {
            row: 1,
            cols: 2..5,
            kind: DrawKind::Fast,
        },
        CellRun {
            row: 3,
            cols: 0..1,
            kind: DrawKind::Slow,
        },
    ];
    let units = Rect::from_min_max(pos2(0.0, 0.0), pos2(10.0, 10.0));
    let screen = Rect::from_min_max(pos2(100.0, 100.0), pos2(200.0, 200.0));
    let to_screen = RectTransform::from_to(units, screen);
    let ctx = Context::default();
    let mut output = ctx.run_ui(RawInput::default(), |ui| {
        paint(ui.painter(), to_screen, &runs, 0.5);
    });
    // No renderer uploads the font atlas here; dropping it unapplied panics.
    output.textures_delta.clear();
    let meshes: Vec<_> = output
        .shapes
        .into_iter()
        .filter_map(|clipped| match clipped.shape {
            Shape::Mesh(mesh) => Some(mesh),
            _ => None,
        })
        .collect();
    let [mesh] = &meshes[..] else {
        panic!("one mesh expected, got {}", meshes.len());
    };
    for (run, quad) in runs.iter().zip(mesh.vertices.chunks(4)) {
        let rect = to_screen.transform_rect(run.rect());
        let expected = fill_colour(run.kind, 0.5);
        for vertex in quad {
            assert!(rect.expand(f32::EPSILON).contains(vertex.pos), "{vertex:?}");
            assert_eq!(vertex.color, expected);
        }
    }
    assert_eq!(mesh.vertices.len(), 4 * runs.len());
}
