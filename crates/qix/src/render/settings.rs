//! The settings screen: a row per setting with its value, the picked
//! one marked, and the controls under them.

use qix_core::Dir;

use crate::bindings::Control;
use crate::input::{CONFIRM_KEY, PAUSE_KEY};
use crate::render::screen::Screen;
use crate::settings::Settings;
use crate::settings_menu::{ROWS, Row, SettingsMenu};

/// Converts a fraction to a percentage.
const PERCENT: f32 = 100.0;

/// The mark before the picked row, and the blank before the others.
const PICKED: &str = "> ";
const NOT_PICKED: &str = "  ";

/// What the picked control's row shows while its key is captured.
const CAPTURING: &str = "press a key";

/// The settings screen of `menu`.
pub fn settings_screen(menu: &SettingsMenu) -> Screen {
    Screen {
        heading: "SETTINGS".to_owned(),
        table: rows(menu),
        lines: help(menu),
    }
}

/// A row per setting, its name padded so the values line up in a
/// monospace font.
fn rows(menu: &SettingsMenu) -> Vec<String> {
    let width = ROWS.iter().map(|&row| label(row).len()).max().unwrap_or(0);
    ROWS.iter()
        .map(|&row| {
            let picked = row == menu.selected();
            let mark = if picked { PICKED } else { NOT_PICKED };
            let value = if picked && menu.is_capturing() {
                CAPTURING.to_owned()
            } else {
                value(menu.settings(), row)
            };
            format!("{mark}{:width$}  {value}", label(row))
                .trim_end()
                .to_owned()
        })
        .collect()
}

/// The name of `row`.
fn label(row: Row) -> &'static str {
    match row {
        Row::Threshold => "Claim threshold",
        Row::StartingLives => "Starting lives",
        Row::Scale => "Window scale",
        Row::Key(control) => control.name(),
        Row::Defaults => "Restore defaults",
        Row::Back => "Back",
    }
}

/// The value of `row` in `settings`; empty for the rows without one.
fn value(settings: &Settings, row: Row) -> String {
    match row {
        Row::Threshold => format!("{:.0}%", settings.threshold.fraction() * PERCENT),
        Row::StartingLives => settings.starting_lives.get().to_string(),
        Row::Scale => format!("x{:.1}", settings.scale.get()),
        Row::Key(control) => settings.keys.key(control).name().to_owned(),
        Row::Defaults | Row::Back => String::new(),
    }
}

/// The controls of the screen, in the keys of `menu`'s settings.
fn help(menu: &SettingsMenu) -> Vec<String> {
    if menu.is_capturing() {
        return vec![format!("{}: cancel", PAUSE_KEY.name())];
    }
    let key = |dir| menu.settings().keys.key(Control::of_dir(dir)).name();
    vec![
        format!("{}/{}: pick", key(Dir::Up), key(Dir::Down)),
        format!("{}/{}: change", key(Dir::Left), key(Dir::Right)),
        format!("{}: rebind the key", CONFIRM_KEY.name()),
        format!("{}: back", PAUSE_KEY.name()),
    ]
}
