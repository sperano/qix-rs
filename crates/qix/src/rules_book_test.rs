use qix_core::tuning::REPEAT_DELAY_TICKS;
use qix_core::{Dir, InputState};

use crate::rules_book::{PAGES, Page, RulesBook};

const CONFIRM: InputState = InputState {
    dir: None,
    fast: false,
    slow: false,
    confirm: true,
    pause: false,
};

const PAUSE: InputState = InputState {
    dir: None,
    fast: false,
    slow: false,
    confirm: false,
    pause: true,
};

/// Holding `dir`.
fn holding(dir: Dir) -> InputState {
    InputState {
        dir: Some(dir),
        ..InputState::default()
    }
}

/// Presses `dir` once: one tick held, one released; the book stays
/// open.
fn press(book: &mut RulesBook, dir: Dir) {
    assert!(!book.tick(&holding(dir)));
    assert!(!book.tick(&InputState::default()));
}

/// A book open at its last page.
fn at_last_page() -> RulesBook {
    let mut book = RulesBook::new();
    for _ in 1..PAGES.len() {
        press(&mut book, Dir::Right);
    }
    assert!(book.is_last_page());
    book
}

#[test]
fn a_new_book_opens_at_the_first_page() {
    let book = RulesBook::new();

    assert_eq!(book.page(), Page::HowToPlay);
    assert_eq!(book.page_index(), 0);
    assert!(!book.is_last_page());
}

#[test]
fn right_turns_to_each_page_in_order_and_stops_at_the_last() {
    let mut book = RulesBook::new();

    press(&mut book, Dir::Right);
    assert_eq!(book.page(), Page::Dangers);
    press(&mut book, Dir::Right);
    assert_eq!(book.page(), Page::Scoring);
    assert_eq!(book.page_index(), 2);
    press(&mut book, Dir::Right);

    assert_eq!(book.page(), Page::Scoring);
    assert!(book.is_last_page());
}

#[test]
fn left_turns_back_and_stops_at_the_first() {
    let mut book = at_last_page();

    press(&mut book, Dir::Left);
    assert_eq!(book.page(), Page::Dangers);
    press(&mut book, Dir::Left);
    press(&mut book, Dir::Left);

    assert_eq!(book.page(), Page::HowToPlay);
}

#[test]
fn up_and_down_do_not_turn_the_page() {
    let mut book = RulesBook::new();
    press(&mut book, Dir::Right);

    press(&mut book, Dir::Up);
    press(&mut book, Dir::Down);

    assert_eq!(book.page(), Page::Dangers);
}

#[test]
fn a_held_direction_turns_one_page_until_it_repeats() {
    let mut book = RulesBook::new();

    for _ in 0..REPEAT_DELAY_TICKS {
        assert!(!book.tick(&holding(Dir::Right)));
    }
    assert_eq!(book.page(), Page::Dangers);
    assert!(!book.tick(&holding(Dir::Right)));

    assert_eq!(book.page(), Page::Scoring);
}

#[test]
fn confirm_turns_to_the_next_page() {
    let mut book = RulesBook::new();

    assert!(!book.tick(&CONFIRM));

    assert_eq!(book.page(), Page::Dangers);
}

#[test]
fn confirm_on_the_last_page_closes_the_book() {
    let mut book = at_last_page();

    assert!(book.tick(&CONFIRM));
}

#[test]
fn pause_closes_the_book_on_any_page() {
    let mut book = RulesBook::new();
    assert!(book.tick(&PAUSE));

    press(&mut book, Dir::Right);
    assert!(book.tick(&PAUSE));
}

#[test]
fn no_input_keeps_the_book_open_on_its_page() {
    let mut book = RulesBook::new();
    press(&mut book, Dir::Right);

    for _ in 0..REPEAT_DELAY_TICKS {
        assert!(!book.tick(&InputState::default()));
    }

    assert_eq!(book.page(), Page::Dangers);
}

/// Confirm pressed with `dir` newly held.
fn confirm_holding(dir: Dir) -> InputState {
    InputState {
        confirm: true,
        ..holding(dir)
    }
}

#[test]
fn confirm_with_a_new_direction_turns_one_page() {
    let mut book = RulesBook::new();

    assert!(!book.tick(&confirm_holding(Dir::Right)));

    assert_eq!(book.page(), Page::Dangers);
}

#[test]
fn confirm_with_a_new_direction_does_not_close_the_book_early() {
    let mut book = RulesBook::new();
    press(&mut book, Dir::Right);

    assert!(!book.tick(&confirm_holding(Dir::Right)));

    assert_eq!(book.page(), Page::Scoring);
}

#[test]
fn confirm_with_left_turns_forward() {
    let mut book = at_last_page();
    press(&mut book, Dir::Left);

    assert!(!book.tick(&confirm_holding(Dir::Left)));

    assert_eq!(book.page(), Page::Scoring);
}
