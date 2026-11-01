//! Frame time of [`show`] at 1080p on nearly full boards, against
//! [`FRAME_BUDGET_MS`]: the CPU's share, building the shapes and
//! tessellating them; the GPU's is not measured. Timing depends on the build and the machine, so
//! the tests are ignored by default; run them in release:
//!
//! ```sh
//! cargo nextest run -p qix --release --run-ignored only --no-capture -E 'test(frame_time)'
//! ```

use std::time::{Duration, Instant};

use eframe::egui::{self, Pos2, Rect, Sense, vec2};
use qix_core::tuning::{BOARD_H, BOARD_W};
use qix_core::{Board, Cell, DrawKind, Game, Qix, Rng, Vec2};

use crate::playfield::{Scene, show, to_screen};
use crate::render;
use crate::render::board::{CellRun, claimed_runs};
use crate::render::claimed::ClaimedTexture;
use crate::render::hud;

/// The most the mean frame may take on the CPU, building the shapes and
/// tessellating them, averaged over the measured frames, in milliseconds.
const FRAME_BUDGET_MS: f64 = 4.0;

/// Size of the window the frames are painted in: 1080p at one pixel per
/// point.
const FRAME_SIZE: egui::Vec2 = vec2(1920.0, 1080.0);

/// Frames painted before the timing starts, so the font atlas and the
/// allocations settle.
const WARMUP_FRAMES: u32 = 30;

/// Frames timed per measurement.
const MEASURED_FRAMES: u32 = 300;

/// Seed of every game measured.
const SEED: u64 = 1;

/// Side of the free square left at the centre of a nearly full board, in
/// cells: about 3% of the board (576 of 19200 cells), around the Qix.
const FREE_SIDE: u16 = 24;

/// Width of the columns of one kind on [`banded_board`], in cells.
const BAND_WIDTH: u16 = 10;

/// Milliseconds in a second.
const MS_PER_S: f64 = 1000.0;

/// The mean and the slowest of a measurement's frames, in milliseconds.
#[derive(Clone, Copy, Debug)]
struct FrameTime {
    mean: f64,
    max: f64,
}

/// A full-size board claimed everywhere but a [`FREE_SIDE`] square at its
/// centre, each claimed cell of the kind `kind_at(x, y)`.
fn nearly_full_board(kind_at: impl Fn(u16, u16) -> DrawKind) -> Board {
    let mut board = Board::new(BOARD_W, BOARD_H);
    let free_x = (BOARD_W - FREE_SIDE) / 2..(BOARD_W + FREE_SIDE) / 2;
    let free_y = (BOARD_H - FREE_SIDE) / 2..(BOARD_H + FREE_SIDE) / 2;
    for y in 0..BOARD_H {
        for x in 0..BOARD_W {
            if !(free_x.contains(&x) && free_y.contains(&y)) {
                board.set_cell(x, y, Cell::Claimed(kind_at(x, y)));
            }
        }
    }
    board
}

/// A nearly full board claimed in [`BAND_WIDTH`] columns of alternating
/// kinds: a few runs per row, as play leaves it.
fn banded_board() -> Board {
    nearly_full_board(|x, _| kind((x / BAND_WIDTH).is_multiple_of(2)))
}

/// A nearly full board claimed in one-cell columns of alternating kinds,
/// as claiming strip after strip along a side leaves it: the most runs a
/// row can be left with in play.
fn striped_board() -> Board {
    nearly_full_board(|x, _| kind(x.is_multiple_of(2)))
}

fn kind(fast: bool) -> DrawKind {
    if fast { DrawKind::Fast } else { DrawKind::Slow }
}

/// A game on `board` with one Qix in the free square at its centre.
fn game_on(board: Board) -> Game {
    let centre = Vec2::new(f32::from(BOARD_W), f32::from(BOARD_H)) * 0.5;
    let half = Vec2::new(f32::from(FREE_SIDE) / 4.0, 0.0);
    let qix = Qix::new(centre - half, centre + half, Vec2::new(0.5, 0.0), 0.0);
    Game::with_board(board, vec![qix], Vec::new(), Rng::new(SEED))
}

/// Times [`MEASURED_FRAMES`] headless frames painting with `paint`, each
/// run through egui and tessellated, after [`WARMUP_FRAMES`] untimed.
fn measure(mut paint: impl FnMut(&mut egui::Ui)) -> FrameTime {
    let ctx = egui::Context::default();
    let mut frame = || {
        let input = egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, FRAME_SIZE)),
            ..egui::RawInput::default()
        };
        let start = Instant::now();
        let mut output = ctx.run_ui(input, &mut paint);
        let primitives = ctx.tessellate(output.shapes, output.pixels_per_point);
        let elapsed = start.elapsed();
        // No renderer uploads the font atlas here; dropping it unapplied panics.
        output.textures_delta.clear();
        assert!(!primitives.is_empty(), "nothing to render");
        elapsed
    };
    (0..WARMUP_FRAMES).for_each(|_| {
        frame();
    });
    let times: Vec<Duration> = (0..MEASURED_FRAMES).map(|_| frame()).collect();
    let ms = |time: Duration| time.as_secs_f64() * MS_PER_S;
    FrameTime {
        mean: ms(times.iter().sum()) / f64::from(MEASURED_FRAMES),
        max: times.iter().copied().map(ms).fold(0.0, f64::max),
    }
}

/// The frame time of painting only `game`'s board, the claimed cells
/// included, over the whole window.
fn board_time(game: &Game) -> FrameTime {
    let mut claimed = ClaimedTexture::default();
    measure(|ui| {
        let texture = claimed.texture(ui.ctx(), game.board());
        let (response, painter) = ui.allocate_painter(ui.available_size(), Sense::hover());
        render::board::paint(&painter, to_screen(response.rect), game, texture, None);
    })
}

/// The frame time of the whole playfield showing, frame after frame, the
/// next of `games`, with the fill animation of `claim` over its board.
/// More than one game redraws the claimed cells' texture every frame.
fn frame_time(games: &[&Game], claim: Option<&[CellRun]>) -> FrameTime {
    let mut claimed = ClaimedTexture::default();
    let mut games = games.iter().cycle();
    measure(|ui| {
        let Some(&game) = games.next() else { return };
        let hud = hud::hud(game, 0);
        let scene = Scene {
            game,
            claimed: claimed.texture(ui.ctx(), game.board()),
            hud: &hud,
            claim: claim.map(|runs| (runs, 0.5)),
            screen: None,
            overlay: true,
        };
        show(ui, &scene);
    })
}

/// Prints `time`, labelled, and checks it is within the budget on
/// average.
fn report(label: &str, time: FrameTime) {
    println!(
        "{label}: mean {:.3} ms, max {:.3} ms (budget {FRAME_BUDGET_MS} ms)",
        time.mean, time.max
    );
    assert!(
        time.mean <= FRAME_BUDGET_MS,
        "{label} over budget: {time:?}"
    );
}

/// Measures painting `board`: the board alone, the whole frame, the whole
/// frame animating a claim of the nearly full area (the free square
/// excluded), and the whole frame redrawing the claimed cells' texture
/// every frame, as on a claim.
fn measure_board(name: &str, board: Board) {
    let game = game_on(board);
    let empty = game_on(Board::new(BOARD_W, BOARD_H));
    let claim = claimed_runs(&nearly_full_board(|_, _| DrawKind::Fast));
    println!("{name}: {} claimed runs", claimed_runs(game.board()).len());
    report(&format!("{name}, board only"), board_time(&game));
    report(&format!("{name}, frame"), frame_time(&[&game], None));
    report(
        &format!("{name}, frame with claim fill"),
        frame_time(&[&game], Some(&claim)),
    );
    report(
        &format!("{name}, frame redrawing the texture"),
        frame_time(&[&game, &empty], None),
    );
}

#[test]
#[ignore = "timing: run in release, see the module documentation"]
fn frame_time_of_an_empty_board() {
    measure_board("empty", Board::new(BOARD_W, BOARD_H));
}

#[test]
#[ignore = "timing: run in release, see the module documentation"]
fn frame_time_of_a_nearly_full_banded_board() {
    measure_board("banded", banded_board());
}

#[test]
#[ignore = "timing: run in release, see the module documentation"]
fn frame_time_of_a_nearly_full_striped_board() {
    measure_board("striped", striped_board());
}
