use eframe::egui::{self, Color32, Pos2, Rect, Shape, pos2, vec2};
use qix_core::tuning::{BOARD_W, STARTING_LIVES};
use qix_core::{Board, Cell, ClaimThreshold, DrawKind, Game, MeterExpiry, Qix, Rng, SparxMeter};

use crate::palette;
use crate::render::hud::{self, Hud, MAX_LIFE_MARKERS, Meter, column_rects, hud};

/// Seed of every game the tests play.
const SEED: u64 = 1207;

/// Ticks of the meter the tests count down.
const METER_TICKS: u32 = 4;

/// Widths of the strips the tests paint, in points: the default window's
/// and the smallest window's.
const STRIP_WIDTHS: [f32; 2] = [944.0, 320.0];

/// Tolerance for comparing screen coordinates, in points.
const EPSILON: f32 = 1e-3;

/// The strip `width` points wide, at the HUD's height for a playfield
/// that wide.
fn strip(width: f32) -> Rect {
    let height = width * hud::HEIGHT / f32::from(BOARD_W);
    Rect::from_min_size(pos2(10.0, 20.0), vec2(width, height))
}

/// A game on a `width x 1` board with its first `claimed` cells claimed.
fn claimed_game(width: u16, claimed: u16) -> Game {
    let mut board = Board::new(width, 1);
    for x in 0..claimed {
        board.set_cell(x, 0, Cell::Claimed(DrawKind::Fast));
    }
    let mut rng = Rng::new(SEED);
    let qix = Qix::spawn(&board, &mut rng);
    Game::with_board(board, vec![qix], Vec::new(), rng)
}

/// A HUD whose every value is as wide as a game makes it.
fn widest_hud() -> Hud {
    Hud {
        score: "99,999,999".to_owned(),
        high_score: "99,999,999".to_owned(),
        level: "99".to_owned(),
        lives: MAX_LIFE_MARKERS + 9,
        claimed: "100.0% / 90%".to_owned(),
        meter: Meter {
            fraction: 1.0,
            next: Some(MeterExpiry::Super),
        },
        multiplier: Some("x9".to_owned()),
    }
}

/// The shapes of one headless egui frame painting `hud` in `strip`.
fn painted(hud: &Hud, strip: Rect) -> Vec<Shape> {
    let ctx = egui::Context::default();
    let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
        hud::paint(ui.painter(), strip, hud);
    });
    // No renderer uploads the font atlas here; dropping it unapplied panics.
    output.textures_delta.clear();
    output
        .shapes
        .into_iter()
        .map(|clipped| clipped.shape)
        .collect()
}

fn texts(shapes: &[Shape]) -> Vec<String> {
    let galleys = shapes.iter().filter_map(|shape| match shape {
        Shape::Text(text) => Some(text.galley.text().to_owned()),
        _ => None,
    });
    galleys.collect()
}

fn polygons(shapes: &[Shape]) -> usize {
    let closed = shapes
        .iter()
        .filter(|s| matches!(s, Shape::Path(p) if p.closed));
    closed.count()
}

/// The rectangles of `shapes` filled with `color`.
fn filled(shapes: &[Shape], color: Color32) -> Vec<Rect> {
    let rects = shapes.iter().filter_map(|shape| match shape {
        Shape::Rect(rect) if rect.fill == color => Some(rect.rect),
        _ => None,
    });
    rects.collect()
}

#[test]
fn a_new_game_shows_no_points_its_lives_level_and_a_full_meter() {
    let hud = hud(&Game::new(Rng::new(SEED)), 0);
    assert_eq!(
        hud,
        Hud {
            score: "0".to_owned(),
            high_score: "0".to_owned(),
            level: "1".to_owned(),
            lives: STARTING_LIVES,
            claimed: "0.0% / 75%".to_owned(),
            meter: Meter {
                fraction: 1.0,
                next: Some(MeterExpiry::Release),
            },
            multiplier: None,
        }
    );
}

#[test]
fn the_high_score_has_thousands_separators() {
    let hud = hud(&Game::new(Rng::new(SEED)), 1_234_567);
    assert_eq!(hud.high_score, "1,234,567");
}

#[test]
fn the_claimed_share_is_rounded_down_to_a_tenth() {
    // Two thirds: 66.67% rounded, 66.6% down.
    let hud = hud(&claimed_game(3, 2), 0);
    assert_eq!(hud.claimed, "66.6% / 75%");
}

#[test]
fn the_claimed_share_is_shown_against_the_game_s_threshold() {
    let mut game = claimed_game(4, 1);
    game.set_claim_threshold(ClaimThreshold::new(0.5));
    assert_eq!(hud(&game, 0).claimed, "25.0% / 50%");
}

#[test]
fn the_multiplier_shows_only_above_one() {
    let mut game = Game::new(Rng::new(SEED));
    game.set_multiplier(3);
    assert_eq!(hud(&game, 0).multiplier.as_deref(), Some("x3"));
}

#[test]
fn the_meter_shrinks_twice_then_is_spent() {
    let mut meter = SparxMeter::new(METER_TICKS);
    meter.tick();
    let release = Meter::from(meter);
    for _ in 1..METER_TICKS {
        meter.tick();
    }
    let super_sparx = Meter::from(meter);
    for _ in 0..METER_TICKS {
        meter.tick();
    }
    let spent = Meter::from(meter);

    let three_quarters = 0.75;
    assert_eq!(release.fraction, three_quarters);
    assert_eq!(release.next, Some(MeterExpiry::Release));
    assert_eq!(super_sparx.fraction, 1.0);
    assert_eq!(super_sparx.next, Some(MeterExpiry::Super));
    assert_eq!(spent.fraction, 0.0);
    assert_eq!(spent.next, None);
}

#[test]
fn each_field_is_named_and_the_multiplier_only_when_shown() {
    let game = Game::new(Rng::new(SEED));
    let names = ["SCORE", "HIGH SCORE", "LEVEL", "LIVES", "CLAIMED", "SPARX"];
    let shapes = painted(&hud(&game, 0), strip(STRIP_WIDTHS[0]));
    let shown = texts(&shapes);
    for name in names {
        assert!(shown.contains(&name.to_owned()), "{name}: {shown:?}");
    }
    assert!(!shown.contains(&"MULTIPLIER".to_owned()), "{shown:?}");
    let shown = texts(&painted(&widest_hud(), strip(STRIP_WIDTHS[0])));
    assert!(shown.contains(&"MULTIPLIER".to_owned()), "{shown:?}");
}

#[test]
fn each_life_is_a_marker() {
    let hud = hud(&Game::new(Rng::new(SEED)), 0);
    let shapes = painted(&hud, strip(STRIP_WIDTHS[0]));
    assert_eq!(polygons(&shapes), STARTING_LIVES as usize);
}

#[test]
fn as_many_markers_as_lives_up_to_the_most_shown() {
    let mut hud = widest_hud();
    for lives in 0..=MAX_LIFE_MARKERS {
        hud.lives = lives;
        let shapes = painted(&hud, strip(STRIP_WIDTHS[0]));
        assert_eq!(polygons(&shapes), lives as usize);
    }
}

#[test]
fn more_lives_show_as_one_marker_and_their_count() {
    let hud = widest_hud();
    let shapes = painted(&hud, strip(STRIP_WIDTHS[0]));
    assert_eq!(polygons(&shapes), 1);
    let count = format!("x{}", hud.lives);
    assert!(texts(&shapes).contains(&count), "{count}");
}

#[test]
fn the_meter_fills_its_share_of_the_bar_in_the_colour_of_its_expiry() {
    let mut hud = widest_hud();
    hud.meter = Meter {
        fraction: 0.5,
        next: Some(MeterExpiry::Release),
    };
    let shapes = painted(&hud, strip(STRIP_WIDTHS[0]));
    let fills = filled(&shapes, palette::meter(MeterExpiry::Release));
    let [fill] = fills[..] else {
        panic!("{shapes:?}");
    };
    let frame = shapes.iter().find_map(|shape| match shape {
        Shape::Rect(rect) if rect.fill == Color32::TRANSPARENT => Some(rect.rect),
        _ => None,
    });
    let frame = frame.expect("no outline");
    assert!((fill.width() - frame.width() / 2.0).abs() < EPSILON);
    assert!((fill.left() - frame.left()).abs() < EPSILON);
}

#[test]
fn a_spent_meter_is_an_empty_bar() {
    let mut hud = widest_hud();
    hud.meter = Meter {
        fraction: 0.0,
        next: None,
    };
    let shapes = painted(&hud, strip(STRIP_WIDTHS[0]));
    for expiry in [MeterExpiry::Release, MeterExpiry::Super] {
        assert!(filled(&shapes, palette::meter(expiry)).is_empty());
    }
}

#[test]
fn the_widest_values_fit_their_columns_at_every_size() {
    for width in STRIP_WIDTHS {
        let strip = strip(width);
        let columns = column_rects(strip);
        for shape in painted(&widest_hud(), strip) {
            let bounds = shape.visual_bounding_rect();
            let fits = columns
                .iter()
                .any(|c| c.expand(EPSILON).contains_rect(bounds));
            let what = match &shape {
                Shape::Text(text) => text.galley.text().to_owned(),
                _ => "a shape".to_owned(),
            };
            assert!(fits, "{width}: {what} at {bounds:?} overflows its column");
        }
    }
}

#[test]
fn the_columns_span_the_strip_side_by_side() {
    let strip = strip(STRIP_WIDTHS[0]);
    let columns = column_rects(strip);
    let mut left: Pos2 = strip.left_top();
    for column in columns {
        assert!((column.left_top() - left).length() < EPSILON);
        assert!((column.height() - strip.height()).abs() < EPSILON);
        left = column.right_top();
    }
    assert!((left - strip.right_top()).length() < EPSILON);
}
