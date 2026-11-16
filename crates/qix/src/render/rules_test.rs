use eframe::egui::Key;
use qix_core::{ClaimThreshold, Dir, InputState, StartingLives};

use crate::bindings::{Control, KeyBindings};
use crate::render::rules::rules_screen;
use crate::rules_book::RulesBook;
use crate::settings::Settings;

/// The book turned `pages` pages on.
fn book_at(pages: usize) -> RulesBook {
    let mut book = RulesBook::new();
    let right = InputState {
        dir: Some(Dir::Right),
        ..InputState::default()
    };
    for _ in 0..pages {
        book.tick(&right);
        book.tick(&InputState::default());
    }
    book
}

/// Settings other than the default ones in every value the rules name.
fn edited() -> Settings {
    let mut keys = KeyBindings::default();
    for (control, key) in [
        (Control::Up, Key::W),
        (Control::Down, Key::S),
        (Control::Left, Key::A),
        (Control::Right, Key::D),
        (Control::Fast, Key::J),
        (Control::Slow, Key::K),
    ] {
        assert!(keys.bind(control, key));
    }
    Settings {
        threshold: ClaimThreshold::new(0.6),
        starting_lives: StartingLives::new(5),
        keys,
        ..Settings::default()
    }
}

#[test]
fn the_first_page_explains_how_to_play() {
    let screen = rules_screen(&book_at(0), &Settings::default());

    assert_eq!(screen.heading, "HOW TO PLAY");
    assert!(screen.table.is_empty());
    assert_eq!(
        screen.lines,
        [
            "Claim 75% of the field to clear a level.",
            "Move along the border with Up/Down/Left/Right.",
            "Hold Z or X while moving to draw",
            "a stix into the open field. Reach the border",
            "again to claim every area without a Qix.",
            "Stop while drawing and a fuse burns along",
            "the stix: it kills you when it reaches you.",
            "",
            "Page 1/3",
            "Left/Right: turn the page",
            "Enter: next page",
            "Escape: back",
        ]
    );
}

#[test]
fn the_second_page_explains_the_dangers() {
    let screen = rules_screen(&book_at(1), &Settings::default());

    assert_eq!(screen.heading, "DANGERS");
    assert_eq!(
        screen.lines,
        [
            "The Qix kills you if it touches your stix",
            "or you while you draw. From level 3, two Qix.",
            "Sparx run along the border and kill on contact.",
            "When the Sparx meter runs out, 2 more",
            "Sparx appear; when it runs out again, they",
            "turn Super and chase you along your stix.",
            "Losing a life refills the meter, and Sparx",
            "cannot hurt you for 2 seconds after.",
            "",
            "Page 2/3",
            "Left/Right: turn the page",
            "Enter: next page",
            "Escape: back",
        ]
    );
}

#[test]
fn the_last_page_explains_the_scoring_and_confirm_goes_back() {
    let screen = rules_screen(&book_at(2), &Settings::default());

    assert_eq!(screen.heading, "SCORING");
    assert_eq!(
        screen.lines,
        [
            "A claimed cell scores 10 x the multiplier,",
            "x2 when drawn slow.",
            "Each percent over 75% adds 1,000 bonus points.",
            "Split two Qix apart to clear the level at once,",
            "with no bonus: the multiplier rises, up to x9.",
            "You start with 3 lives and earn one more",
            "every 30,000 points, up to 9.",
            "",
            "Page 3/3",
            "Left/Right: turn the page",
            "Enter: back",
            "Escape: back",
        ]
    );
}

#[test]
fn the_rules_follow_the_settings_in_force() {
    let settings = edited();

    let first = rules_screen(&book_at(0), &settings).lines;
    let last = rules_screen(&book_at(2), &settings).lines;

    assert_eq!(first[0], "Claim 60% of the field to clear a level.");
    assert_eq!(first[1], "Move along the border with W/S/A/D.");
    assert_eq!(first[2], "Hold J or K while moving to draw");
    assert_eq!(first[9], "A/D: turn the page");
    assert_eq!(last[2], "Each percent over 60% adds 1,000 bonus points.");
    assert_eq!(last[5], "You start with 5 lives and earn one more");
}

#[test]
fn a_single_starting_life_reads_in_the_singular() {
    let settings = Settings {
        starting_lives: StartingLives::new(1),
        ..Settings::default()
    };

    let lines = rules_screen(&book_at(2), &settings).lines;

    assert_eq!(lines[5], "You start with 1 life and earn one more");
}
