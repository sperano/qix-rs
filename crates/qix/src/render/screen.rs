//! The screens over the playfield outside play: the title with the
//! controls, the pause, the end of a level and the end of the game.

use std::sync::Arc;

use eframe::egui::{FontId, Galley, Painter, Rect, pos2, vec2};
use qix_core::tuning::SLOW_DRAW_FACTOR;
use qix_core::{Game, GameState, Session};

use crate::input::{CONFIRM_KEY, DIR_KEYS, FAST_KEY, OVERLAY_KEY, PAUSE_KEY, SLOW_KEY};
use crate::palette;

/// Size of a screen's heading, in points.
const HEADING_SIZE: f32 = 40.0;

/// Size of the lines under the heading, in points.
const LINE_SIZE: f32 = 18.0;

/// Space between the heading and the first line, in points.
const HEADING_GAP: f32 = 16.0;

/// Space between a screen's text and the edge of its panel, in points.
const PADDING: f32 = 24.0;

/// Corner radius of a screen's panel, in points.
const CORNER_RADIUS: f32 = 8.0;

/// Converts a fraction to a percentage.
const PERCENT: f32 = 100.0;

/// The text of a screen: a heading over lines, all centred.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Screen {
    pub heading: String,
    pub lines: Vec<String>,
}

/// The screen `session` shows over the playfield, none while playing.
pub fn screen(session: &Session) -> Option<Screen> {
    let game = session.game();
    match session.state() {
        GameState::Title => Some(title()),
        GameState::Playing => None,
        GameState::Paused => Some(paused()),
        GameState::LevelComplete {
            fraction,
            bonus,
            split,
        } => Some(level_complete(game, fraction, bonus, split)),
        GameState::GameOver => Some(game_over(game)),
    }
}

/// The title screen, with the controls.
pub fn title() -> Screen {
    let dirs: Vec<_> = DIR_KEYS.iter().map(|(key, _)| key.name()).collect();
    Screen {
        heading: "QIX".to_owned(),
        lines: vec![
            format!("{}: move", dirs.join("/")),
            format!("hold {} while moving: draw fast", FAST_KEY.name()),
            format!(
                "hold {} while moving: draw slow, x{SLOW_DRAW_FACTOR} points",
                SLOW_KEY.name()
            ),
            format!("{}: pause", PAUSE_KEY.name()),
            format!("{}: debug overlay", OVERLAY_KEY.name()),
            String::new(),
            format!("Press {} to start", CONFIRM_KEY.name()),
        ],
    }
}

/// The pause overlay.
pub fn paused() -> Screen {
    Screen {
        heading: "PAUSED".to_owned(),
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
        format!("Bonus {bonus}"),
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
        lines,
    }
}

/// The end of `game`, with its final score.
pub fn game_over(game: &Game) -> Screen {
    Screen {
        heading: "GAME OVER".to_owned(),
        lines: vec![
            format!("Final score {}", game.score().points()),
            String::new(),
            format!("Press {} to continue", CONFIRM_KEY.name()),
        ],
    }
}

/// Paints `screen` over `field`, the playfield on screen: the field
/// dimmed, still visible, and the text centred on a panel.
pub fn paint(painter: &Painter, field: Rect, screen: &Screen) {
    painter.rect_filled(field, 0.0, palette::SCREEN_DIM);
    let rows = layout(painter, screen);
    let width = rows
        .iter()
        .map(|(galley, _)| galley.size().x)
        .fold(0.0, f32::max);
    let height = rows.iter().map(|(galley, gap)| galley.size().y + gap).sum();
    let text = Rect::from_center_size(field.center(), vec2(width, height));
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
/// it: the heading, then each line.
fn layout(painter: &Painter, screen: &Screen) -> Vec<(Arc<Galley>, f32)> {
    let heading = painter.layout_no_wrap(
        screen.heading.clone(),
        FontId::proportional(HEADING_SIZE),
        palette::SCREEN_HEADING,
    );
    let lines = screen.lines.iter().map(|line| {
        let font = FontId::proportional(LINE_SIZE);
        let galley = painter.layout_no_wrap(line.clone(), font, palette::SCREEN_TEXT);
        (galley, 0.0)
    });
    std::iter::once((heading, HEADING_GAP))
        .chain(lines)
        .collect()
}
