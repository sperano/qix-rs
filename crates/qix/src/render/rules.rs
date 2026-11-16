//! The rules screen: the page shown, with the rules the settings put in
//! force and the keys they bind, then where it is in the book and the
//! controls under it.

use qix_core::tuning::{
    BONUS_PER_PERCENT, EXTRA_LIFE_EVERY, EXTRA_SPARX, MAX_LIVES, MAX_MULTIPLIER, POINTS_PER_CELL,
    RESPAWN_GRACE_TICKS, SLOW_DRAW_FACTOR, TICK_HZ, TWO_QIX_FROM_LEVEL,
};

use crate::bindings::{Control, KeyBindings};
use crate::input::{CONFIRM_KEY, PAUSE_KEY};
use crate::numbers::thousands;
use crate::render::screen::Screen;
use crate::rules_book::{PAGES, Page, RulesBook};
use crate::settings::Settings;

/// Converts a fraction to a percentage.
const PERCENT: f32 = 100.0;

/// The rules screen of `book`, with the rules and keys of `settings`.
pub fn rules_screen(book: &RulesBook, settings: &Settings) -> Screen {
    let page = book.page();
    let mut lines = text(page, settings);
    lines.push(String::new());
    lines.extend(help(book, &settings.keys));
    Screen {
        heading: heading(page).to_owned(),
        table: Vec::new(),
        lines,
    }
}

/// The heading of `page`.
fn heading(page: Page) -> &'static str {
    match page {
        Page::HowToPlay => "HOW TO PLAY",
        Page::Dangers => "DANGERS",
        Page::Scoring => "SCORING",
    }
}

/// The rules `page` explains, with the rules and keys of `settings`.
fn text(page: Page, settings: &Settings) -> Vec<String> {
    match page {
        Page::HowToPlay => how_to_play(settings),
        Page::Dangers => dangers(),
        Page::Scoring => scoring(settings),
    }
}

/// The claim threshold of `settings`, in whole percent.
fn threshold_percent(settings: &Settings) -> String {
    format!("{:.0}%", settings.threshold.fraction() * PERCENT)
}

/// The goal, and how the marker draws and claims.
fn how_to_play(settings: &Settings) -> Vec<String> {
    let key = |control| settings.keys.key(control).name();
    let dirs: Vec<_> = Control::DIRS
        .iter()
        .map(|&(control, _)| key(control))
        .collect();
    vec![
        format!(
            "Claim {} of the field to clear a level.",
            threshold_percent(settings)
        ),
        format!("Move along the border with {}.", dirs.join("/")),
        format!(
            "Hold {} or {} while moving to draw",
            key(Control::Fast),
            key(Control::Slow)
        ),
        "a stix into the open field. Reach the border".to_owned(),
        "again to claim every area without a Qix.".to_owned(),
        "Stop while drawing and a fuse burns along".to_owned(),
        "the stix: it kills you when it reaches you.".to_owned(),
    ]
}

/// The Qix and the Sparx.
fn dangers() -> Vec<String> {
    vec![
        "The Qix kills you if it touches your stix".to_owned(),
        format!("or you while you draw. From level {TWO_QIX_FROM_LEVEL}, two Qix."),
        "Sparx run along the border and kill on contact.".to_owned(),
        format!("When the Sparx meter runs out, {EXTRA_SPARX} more"),
        "Sparx appear; when it runs out again, they".to_owned(),
        "turn Super and chase you along your stix.".to_owned(),
        "Losing a life refills the meter, and Sparx".to_owned(),
        format!(
            "cannot hurt you for {} seconds after.",
            RESPAWN_GRACE_TICKS / TICK_HZ
        ),
    ]
}

/// The points, the bonus, the multiplier and the lives of `settings`.
fn scoring(settings: &Settings) -> Vec<String> {
    vec![
        format!("A claimed cell scores {POINTS_PER_CELL} x the multiplier,"),
        format!("x{SLOW_DRAW_FACTOR} when drawn slow."),
        format!(
            "Each percent over {} adds {} bonus points.",
            threshold_percent(settings),
            thousands(BONUS_PER_PERCENT)
        ),
        "Split two Qix apart to clear the level at once,".to_owned(),
        format!("with no bonus: the multiplier rises, up to x{MAX_MULTIPLIER}."),
        format!(
            "You start with {} and earn one more",
            lives(settings.starting_lives.get())
        ),
        format!(
            "every {} points, up to {MAX_LIVES}.",
            thousands(EXTRA_LIFE_EVERY)
        ),
    ]
}

/// `count` lives, in words: "1 life", "3 lives".
fn lives(count: u32) -> String {
    let noun = if count == 1 { "life" } else { "lives" };
    format!("{count} {noun}")
}

/// Where `book` is, and the controls of the screen in `keys`.
fn help(book: &RulesBook, keys: &KeyBindings) -> Vec<String> {
    let confirm = if book.is_last_page() {
        "back"
    } else {
        "next page"
    };
    vec![
        format!("Page {}/{}", book.page_index() + 1, PAGES.len()),
        format!(
            "{}/{}: turn the page",
            keys.key(Control::Left).name(),
            keys.key(Control::Right).name()
        ),
        format!("{}: {confirm}", CONFIRM_KEY.name()),
        format!("{}: back", PAUSE_KEY.name()),
    ]
}
