//! The HUD: the strip above the playfield with the score, the high score,
//! the lives, the level, the claimed share against the threshold, the
//! Sparx meter and the multiplier.
//!
//! Sizes are fractions of the strip's height, so the HUD scales with the
//! letterboxed playfield and its text fits its columns at every window
//! size.

use eframe::egui::{Align2, FontId, Painter, Pos2, Rect, Shape, Stroke, StrokeKind, pos2, vec2};
use qix_core::tuning::STARTING_MULTIPLIER;
use qix_core::{Game, MeterExpiry, SparxMeter};

use crate::numbers::thousands;
use crate::palette;
use crate::render::board::diamond;

/// Height of the strip, in playfield units.
pub const HEIGHT: f32 = 12.0;

/// Columns of the strip.
const COLUMNS: usize = 7;

/// The columns of the strip, left to right, each with its share of the
/// strip's width: score, high score, level, lives, claimed, Sparx meter,
/// multiplier.
const COLUMN_WEIGHTS: [f32; COLUMNS] = [3.0, 3.0, 2.0, 3.0, 4.0, 3.0, 2.0];

/// Size of a field's name, as a fraction of the strip's height.
const LABEL_SIZE: f32 = 0.22;

/// Top of a field's name, as a fraction of the strip's height.
const LABEL_TOP: f32 = 0.12;

/// Size of a field's value, as a fraction of the strip's height.
const VALUE_SIZE: f32 = 0.30;

/// Centre of a field's value, as a fraction of the strip's height.
const VALUE_CENTRE: f32 = 0.66;

/// Lives drawn as markers; more show as one marker and their count.
pub const MAX_LIFE_MARKERS: u32 = 5;

/// Distance from a life marker's centre to its corners, as a fraction of
/// the strip's height.
const LIFE_RADIUS: f32 = 0.12;

/// Distance between the centres of two life markers, in life radii.
const LIFE_SPACING: f32 = 2.6;

/// Width of the Sparx meter's bar, as a fraction of its column's width.
const METER_WIDTH: f32 = 0.8;

/// Height of the Sparx meter's bar, as a fraction of the strip's height.
const METER_HEIGHT: f32 = 0.2;

/// Width of the Sparx meter's outline, in points.
const METER_FRAME_WIDTH: f32 = 1.0;

/// Converts a fraction to a percentage.
const PERCENT: f64 = 100.0;

/// Tenths in one, to show percentages to a tenth.
const TENTHS: f64 = 10.0;

/// Tenths of a percent added before rounding the claimed share down:
/// absorbs the `f32` rounding of a share lying on a tenth, such as 70 %
/// stored a hair below `0.7`, yet stays well under the share of one cell
/// of a full board.
const TENTHS_ROUNDING_SLACK: f64 = 1e-4;

/// What the HUD shows of one game, as text where it is text.
#[derive(Clone, Debug, PartialEq)]
pub struct Hud {
    pub score: String,
    pub high_score: String,
    pub level: String,
    pub lives: u32,
    /// The claimed share of the board, then the threshold.
    pub claimed: String,
    pub meter: Meter,
    /// The multiplier, none while it is the starting one.
    pub multiplier: Option<String>,
}

/// The Sparx meter's bar.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Meter {
    /// The share of the current countdown left, from 1 (full) to 0.
    pub fraction: f32,
    /// What the countdown leads to, `None` once the meter is spent.
    pub next: Option<MeterExpiry>,
}

impl From<SparxMeter> for Meter {
    fn from(meter: SparxMeter) -> Self {
        Self {
            fraction: meter.remaining() as f32 / meter.length() as f32,
            next: meter.next_expiry(),
        }
    }
}

/// The HUD of `game`, with the session's `high_score`.
pub fn hud(game: &Game, high_score: u32) -> Hud {
    let multiplier = game.multiplier();
    Hud {
        score: thousands(game.score().points()),
        high_score: thousands(high_score),
        level: game.level().to_string(),
        lives: game.lives(),
        claimed: claimed(game),
        meter: game.sparx_meter().into(),
        multiplier: (multiplier > STARTING_MULTIPLIER).then(|| format!("x{multiplier}")),
    }
}

/// The claimed share of `game`'s board and its threshold, in percent:
/// the share rounded down to a tenth, so it never shows the threshold
/// before the board reaches it.
fn claimed(game: &Game) -> String {
    let fraction = f64::from(game.board().claimed_fraction());
    let tenths = (fraction * PERCENT * TENTHS + TENTHS_ROUNDING_SLACK).floor() / TENTHS;
    let threshold = f64::from(game.claim_threshold().fraction()) * PERCENT;
    format!("{tenths:.1}% / {threshold:.0}%")
}

/// The value of one column.
#[derive(Clone, Copy, Debug)]
enum Value<'a> {
    Text(&'a str),
    Lives(u32),
    Meter(Meter),
}

/// The columns of `hud`, left to right, each with its name and value;
/// the multiplier's is empty while there is none.
fn columns(hud: &Hud) -> [Option<(&'static str, Value<'_>)>; COLUMNS] {
    [
        Some(("SCORE", Value::Text(&hud.score))),
        Some(("HIGH SCORE", Value::Text(&hud.high_score))),
        Some(("LEVEL", Value::Text(&hud.level))),
        Some(("LIVES", Value::Lives(hud.lives))),
        Some(("CLAIMED", Value::Text(&hud.claimed))),
        Some(("SPARX", Value::Meter(hud.meter))),
        hud.multiplier
            .as_deref()
            .map(|multiplier| ("MULTIPLIER", Value::Text(multiplier))),
    ]
}

/// The rectangles of the columns of `strip`, left to right.
pub fn column_rects(strip: Rect) -> [Rect; COLUMNS] {
    let total: f32 = COLUMN_WEIGHTS.iter().sum();
    let mut left = strip.min.x;
    COLUMN_WEIGHTS.map(|weight| {
        let width = strip.width() * weight / total;
        let column = Rect::from_min_size(pos2(left, strip.min.y), vec2(width, strip.height()));
        left += width;
        column
    })
}

/// Paints `hud` in `strip`, the rectangle above the playfield on screen.
pub fn paint(painter: &Painter, strip: Rect, hud: &Hud) {
    for (column, field) in column_rects(strip).into_iter().zip(columns(hud)) {
        if let Some((label, value)) = field {
            paint_column(painter, column, label, value);
        }
    }
}

/// Paints the field named `label` with its `value` in `column`.
fn paint_column(painter: &Painter, column: Rect, label: &str, value: Value<'_>) {
    let height = column.height();
    let label_pos = pos2(column.center().x, column.min.y + LABEL_TOP * height);
    let label_font = FontId::proportional(LABEL_SIZE * height);
    painter.text(
        label_pos,
        Align2::CENTER_TOP,
        label,
        label_font,
        palette::HUD_LABEL,
    );
    let centre = pos2(column.center().x, column.min.y + VALUE_CENTRE * height);
    match value {
        Value::Text(text) => {
            paint_value(painter, centre, Align2::CENTER_CENTER, text, height);
        }
        Value::Lives(lives) => paint_lives(painter, centre, lives, height),
        Value::Meter(meter) => paint_meter(painter, centre, column.width(), meter, height),
    }
}

/// Paints `text` as a value anchored by `align` at `pos`, in a strip
/// `height` points high.
fn paint_value(painter: &Painter, pos: Pos2, align: Align2, text: &str, height: f32) {
    let font = FontId::monospace(VALUE_SIZE * height);
    painter.text(pos, align, text, font, palette::HUD_VALUE);
}

/// Paints `lives` centred on `centre`: a row of markers, or past
/// [`MAX_LIFE_MARKERS`] one marker followed by the count.
fn paint_lives(painter: &Painter, centre: Pos2, lives: u32, height: f32) {
    let radius = LIFE_RADIUS * height;
    let spacing = LIFE_SPACING * radius;
    if lives > MAX_LIFE_MARKERS {
        let marker = centre - vec2(spacing / 2.0, 0.0);
        paint_life(painter, marker, radius);
        let count = format!("x{lives}");
        paint_value(painter, centre, Align2::LEFT_CENTER, &count, height);
        return;
    }
    let span = lives.saturating_sub(1) as f32 * spacing;
    let first = centre - vec2(span / 2.0, 0.0);
    for i in 0..lives {
        paint_life(painter, first + vec2(i as f32 * spacing, 0.0), radius);
    }
}

/// Paints one life as a marker centred on `centre`, `radius` points to
/// its corners.
fn paint_life(painter: &Painter, centre: Pos2, radius: f32) {
    painter.add(Shape::convex_polygon(
        diamond(centre, radius).to_vec(),
        palette::HUD_LIFE,
        Stroke::NONE,
    ));
}

/// Paints `meter` as a bar centred on `centre` in a column
/// `column_width` points wide: an outline, and inside it a fill from the
/// left as long as the share of the countdown left.
fn paint_meter(painter: &Painter, centre: Pos2, column_width: f32, meter: Meter, height: f32) {
    let frame = Rect::from_center_size(
        centre,
        vec2(METER_WIDTH * column_width, METER_HEIGHT * height),
    );
    if let Some(next) = meter.next {
        let mut fill = frame;
        fill.set_width(frame.width() * meter.fraction.clamp(0.0, 1.0));
        painter.rect_filled(fill, 0.0, palette::meter(next));
    }
    let stroke = Stroke::new(METER_FRAME_WIDTH, palette::HUD_METER_FRAME);
    painter.rect_stroke(frame, 0.0, stroke, StrokeKind::Inside);
}
