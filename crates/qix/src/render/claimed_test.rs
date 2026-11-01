use eframe::egui::{
    self, Color32, ImageData, Pos2, Rect, Shape, TextureId, TextureOptions, emath::RectTransform,
    epaint::ImageDelta, pos2,
};
use qix_core::{Board, Cell, DrawKind};

use crate::palette;
use crate::render::claimed::{ClaimedTexture, image, paint};

/// Dimensions of the board the tests use.
const W: u16 = 4;
const H: u16 = 3;

/// Screen points per cell in [`paint`]'s test.
const SCALE: f32 = 5.0;

/// The texture [`paint`]'s test paints.
const TEXTURE: TextureId = TextureId::User(1);

/// The index of the texel of cell `(x, y)` in a `W`-wide image.
fn texel(x: u16, y: u16) -> usize {
    usize::from(y) * usize::from(W) + usize::from(x)
}

#[test]
fn a_fresh_board_is_a_transparent_image_of_one_texel_per_cell() {
    let image = image(&Board::new(W, H));
    assert_eq!(image.size, [usize::from(W), usize::from(H)]);
    assert!(image.pixels.iter().all(|&p| p == Color32::TRANSPARENT));
}

#[test]
fn claimed_cells_take_the_fill_of_their_kind_row_by_row() {
    let mut board = Board::new(W, H);
    board.set_cell(1, 0, Cell::Claimed(DrawKind::Fast));
    board.set_cell(3, 2, Cell::Claimed(DrawKind::Slow));

    let image = image(&board);

    assert_eq!(image.pixels[texel(1, 0)], palette::FAST_FILL);
    assert_eq!(image.pixels[texel(3, 2)], palette::SLOW_FILL);
    let claimed = image.pixels.iter().filter(|&&p| p != Color32::TRANSPARENT);
    assert_eq!(claimed.count(), 2);
}

/// Runs one headless frame asking `texture` for `board`'s texture;
/// returns it and the frame's uploads of it.
fn frame(
    ctx: &egui::Context,
    texture: &mut ClaimedTexture,
    board: &Board,
) -> (TextureId, Vec<ImageDelta>) {
    let mut id = None;
    let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
        id = Some(texture.texture(ui.ctx(), board));
    });
    let id = id.unwrap_or_else(|| panic!("the frame did not run"));
    let uploads = output.textures_delta.set.remove(&id).unwrap_or_default();
    // No renderer uploads the textures here; dropping them unapplied panics.
    output.textures_delta.clear();
    (id, uploads.into_vec())
}

/// Checks that `uploads` is one upload replacing the whole texture with
/// `board`'s image, filtered to the nearest texel.
fn assert_uploads_board(uploads: Vec<ImageDelta>, board: &Board) {
    let [upload] = &uploads[..] else {
        panic!("one upload expected, got {}", uploads.len());
    };
    assert_eq!(upload.pos, None, "a partial upload");
    assert_eq!(upload.options, TextureOptions::NEAREST);
    let ImageData::Color(uploaded) = &upload.image;
    let expected = image(board);
    assert_eq!(uploaded.size, expected.size);
    assert_eq!(uploaded.pixels, expected.pixels);
}

#[test]
fn the_texture_is_uploaded_on_the_first_frame_only_while_the_board_stays() {
    let ctx = egui::Context::default();
    let mut texture = ClaimedTexture::default();
    let mut board = Board::new(W, H);
    board.set_cell(3, 2, Cell::Claimed(DrawKind::Slow));

    let (first, uploads) = frame(&ctx, &mut texture, &board);
    assert_uploads_board(uploads, &board);
    let (second, uploads) = frame(&ctx, &mut texture, &board);
    assert_eq!(second, first);
    assert!(uploads.is_empty());
}

#[test]
fn a_claim_redraws_the_same_texture() {
    let ctx = egui::Context::default();
    let mut texture = ClaimedTexture::default();
    let mut board = Board::new(W, H);
    let (first, _) = frame(&ctx, &mut texture, &board);

    board.set_cell(0, 0, Cell::Claimed(DrawKind::Fast));
    let (claimed, uploads) = frame(&ctx, &mut texture, &board);

    assert_eq!(claimed, first);
    assert_uploads_board(uploads, &board);
    assert!(frame(&ctx, &mut texture, &board).1.is_empty());
}

#[test]
fn the_texture_is_painted_once_over_the_whole_board() {
    let cells = Rect::from_min_max(Pos2::ZERO, pos2(f32::from(W), f32::from(H)));
    let screen = Rect::from_min_max(pos2(10.0, 20.0), pos2(10.0, 20.0) + cells.size() * SCALE);
    let ctx = egui::Context::default();
    let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
        paint(
            ui.painter(),
            RectTransform::from_to(cells, screen),
            TEXTURE,
            (W, H),
        );
    });
    output.textures_delta.clear();

    let [shape] = &output.shapes[..] else {
        panic!("one shape expected: {:?}", output.shapes);
    };
    let Shape::Mesh(mesh) = &shape.shape else {
        panic!("not a mesh: {:?}", shape.shape);
    };
    assert_eq!(mesh.texture_id, TEXTURE);
    assert_eq!(mesh.calc_bounds(), screen);
    let corners = [
        (screen.left_top(), pos2(0.0, 0.0)),
        (screen.right_top(), pos2(1.0, 0.0)),
        (screen.left_bottom(), pos2(0.0, 1.0)),
        (screen.right_bottom(), pos2(1.0, 1.0)),
    ];
    for (pos, uv) in corners {
        let vertex = mesh.vertices.iter().find(|v| v.pos == pos);
        let vertex = vertex.unwrap_or_else(|| panic!("no vertex at {pos:?}: {mesh:?}"));
        assert_eq!(vertex.uv, uv, "texture coordinates at {pos:?}");
    }
    assert!(mesh.vertices.iter().all(|v| v.color == Color32::WHITE));
}
