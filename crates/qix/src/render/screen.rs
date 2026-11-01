//! The screens over the playfield outside play: the title with the high
//! scores and the controls, the pause, the end of a level, the end of the
//! game and the entry of a high score.

use std::sync::Arc;

use eframe::egui::{FontId, Galley, Painter, Rect, Vec2, pos2, vec2};
use qix_core::tuning::SLOW_DRAW_FACTOR;
use qix_core::{Game, GameState, HighScores, Session};

use crate::bindings::{Control, KeyBindings};
use crate::input::{CONFIRM_KEY, OPEN_SETTINGS_KEY, OVERLAY_KEY, PAUSE_KEY};
use crate::numbers::thousands;
use crate::palette;
use crate::render::high_score::{high_score_entry, table_rows};

/// Size of a screen's heading, in points.
const HEADING_SIZE: f32 = 40.0;

/// Size of the lines under the heading, in points.
const LINE_SIZE: f32 = 18.0;

/// Space between the heading and the first line, in points.
const HEADING_GAP: f32 = 16.0;

/// Space between the last row of a table and the first line, in points.
const TABLE_GAP: f32 = 16.0;

/// Space between a screen's text and the edge of its panel, in points.
const PADDING: f32 = 24.0;

/// Space between the top of the playfield and a banner's panel, in
/// points.
const BANNER_MARGIN: f32 = 16.0;

/// Corner radius of a screen's panel, in points.
const CORNER_RADIUS: f32 = 8.0;

/// Converts a fraction to a percentage.
const PERCENT: f32 = 100.0;

/// The text of a screen, all centred: a heading, the rows of a table in
/// a monospace font, then lines.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Screen {
    pub heading: String,
    pub table: Vec<String>,
    pub lines: Vec<String>,
}

/// The screen `session` shows over the playfield, none while playing,
/// naming the keys of `keys`.
pub fn screen(session: &Session, keys: &KeyBindings) -> Option<Screen> {
    let game = session.game();
    let high_scores = session.high_scores();
    match session.state() {
        GameState::Title => Some(title(high_scores, keys)),
        GameState::Playing => None,
        GameState::Paused => Some(paused()),
        GameState::LevelComplete {
            fraction,
            bonus,
            split,
        } => Some(level_complete(game, fraction, bonus, split)),
        GameState::GameOver => Some(game_over(game, high_scores)),
        GameState::HighScoreEntry(entry) => Some(high_score_entry(game, high_scores, &entry, keys)),
    }
}

/// The title screen, with `high_scores` and the controls in `keys`.
pub fn title(high_scores: &HighScores, keys: &KeyBindings) -> Screen {
    let dirs: Vec<_> = Control::DIRS
        .iter()
        .map(|&(control, _)| keys.key(control).name())
        .collect();
    let key = |control| keys.key(control).name();
    Screen {
        heading: "QIX".to_owned(),
        table: table_rows(high_scores),
        lines: vec![
            format!("{}: move", dirs.join("/")),
            format!("hold {} while moving: draw fast", key(Control::Fast)),
            format!(
                "hold {} while moving: draw slow, x{SLOW_DRAW_FACTOR} points",
                key(Control::Slow)
            ),
            format!("{}: pause", PAUSE_KEY.name()),
            format!("{}: debug overlay", OVERLAY_KEY.name()),
            format!("{}: settings", OPEN_SETTINGS_KEY.name()),
            String::new(),
            format!("Press {} to start", CONFIRM_KEY.name()),
        ],
    }
}

/// The pause overlay.
pub fn paused() -> Screen {
    Screen {
        heading: "PAUSED".to_owned(),
        table: Vec::new(),
        lines: vec![format!("Press {} to resume", PAUSE_KEY.name())],
    }
}

/// The end of `game`'s level, which claimed `fraction` of the board and
/// scored `bonus`, or ended by a `split`.
pub fn level_complete(game: &Game, fraction: f32, bonus: u32, split: bool) -> Screen {
    let mut lines = Vec::new();
    if split {
        lines.push("The Qix are split apart!".to_owned());
    }
    lines.extend([
        format!("Claimed {:.1}%", fraction * PERCENT),
        format!("Bonus {}", thousands(bonus)),
        format!("Multiplier x{}", game.multiplier()),
        String::new(),
        format!(
            "Press {} for level {}",
            CONFIRM_KEY.name(),
            game.level() + 1
        ),
    ]);
    Screen {
        heading: format!("LEVEL {} COMPLETE", game.level()),
        table: Vec::new(),
        lines,
    }
}

/// The end of `game`, with its final score and whether it makes
/// `high_scores`.
pub fn game_over(game: &Game, high_scores: &HighScores) -> Screen {
    let points = game.score().points();
    let mut lines = vec![format!("Final score {}", thousands(points))];
    if high_scores.qualifies(points) {
        lines.push("A new high score!".to_owned());
    }
    lines.extend([
        String::new(),
        format!("Press {} to continue", CONFIRM_KEY.name()),
    ]);
    Screen {
        heading: "GAME OVER".to_owned(),
        table: Vec::new(),
        lines,
    }
}

/// The banner over the attract-mode demo: the demo plays on under it,
/// and confirm starts a game.
pub fn demo() -> Screen {
    Screen {
        heading: "DEMO".to_owned(),
        table: Vec::new(),
        lines: vec![format!("Press {} to play", CONFIRM_KEY.name())],
    }
}

/// Paints `screen` over `field`, the playfield on screen: the field
/// dimmed, still visible, and the text centred on a panel.
pub fn paint(painter: &Painter, field: Rect, screen: &Screen) {
    painter.rect_filled(field, 0.0, palette::SCREEN_DIM);
    paint_panel(painter, screen, |size| {
        Rect::from_center_size(field.center(), size)
    });
}

/// Paints `screen` as a banner at the top of `field`, the playfield on
/// screen, which is not dimmed: the text on a panel centred
/// horizontally, [`BANNER_MARGIN`] below the field's top.
pub fn paint_banner(painter: &Painter, field: Rect, screen: &Screen) {
    paint_panel(painter, screen, |size| {
        let top = field.min.y + BANNER_MARGIN + PADDING;
        Rect::from_min_size(pos2(field.center().x - size.x / 2.0, top), size)
    });
}

/// Paints `screen`'s text on a panel, the text in the rectangle `place`
/// gives for its size.
fn paint_panel(painter: &Painter, screen: &Screen, place: impl FnOnce(Vec2) -> Rect) {
    let rows = layout(painter, screen);
    let width = rows
        .iter()
        .map(|(galley, _)| galley.size().x)
        .fold(0.0, f32::max);
    let height = rows.iter().map(|(galley, gap)| galley.size().y + gap).sum();
    let text = place(vec2(width, height));
    painter.rect_filled(
        text.expand(PADDING),
        CORNER_RADIUS,
        palette::SCREEN_BACKGROUND,
    );
    let mut top = text.min.y;
    for (galley, gap) in rows {
        let size = galley.size();
        let pos = pos2(text.center().x - size.x / 2.0, top);
        top += size.y + gap;
        painter.galley(pos, galley, palette::SCREEN_TEXT);
    }
}

/// The rows of `screen`'s text, top to bottom, each with the space under
/// it: the heading, each row of the table, then each line.
fn layout(painter: &Painter, screen: &Screen) -> Vec<(Arc<Galley>, f32)> {
    let heading = painter.layout_no_wrap(
        screen.heading.clone(),
        FontId::proportional(HEADING_SIZE),
        palette::SCREEN_HEADING,
    );
    let text =
        |line: &String, font| painter.layout_no_wrap(line.clone(), font, palette::SCREEN_TEXT);
    let last_row = screen.table.len().checked_sub(1);
    let table = screen.table.iter().enumerate().map(|(i, row)| {
        let gap = if Some(i) == last_row { TABLE_GAP } else { 0.0 };
        (text(row, FontId::monospace(LINE_SIZE)), gap)
    });
    let lines = screen
        .lines
        .iter()
        .map(|line| (text(line, FontId::proportional(LINE_SIZE)), 0.0));
    std::iter::once((heading, HEADING_GAP))
        .chain(table)
        .chain(lines)
        .collect()
}
