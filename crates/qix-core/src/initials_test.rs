use crate::tuning::{INITIALS_LEN, INITIALS_REPEAT_DELAY_TICKS, INITIALS_REPEAT_TICKS};
use crate::{Dir, Initials, InitialsEntry, InputState, InvalidInitials};

/// Letters from A to Z: stepping a letter this many times comes back to it.
const LETTER_COUNT: u32 = 26;

/// Repeats [`count_steps`] watches a held direction for.
const REPEATS: u32 = 3;

fn held(dir: Dir) -> InputState {
    InputState {
        dir: Some(dir),
        ..InputState::default()
    }
}

const CONFIRM: InputState = InputState {
    dir: None,
    fast: false,
    slow: false,
    confirm: true,
    pause: false,
};

/// Ticks `entry` with `dir` held for one tick, then released.
fn tap(entry: &mut InitialsEntry, dir: Dir) {
    assert_eq!(entry.tick(&held(dir)), None);
    assert_eq!(entry.tick(&InputState::default()), None);
}

fn initials(text: &str) -> Initials {
    Initials::try_from(text.to_owned()).unwrap()
}

#[test]
fn initials_hold_three_capital_letters() {
    assert_eq!(initials("QIX").letters(), ['Q', 'I', 'X']);
    assert_eq!(initials("QIX").to_string(), "QIX");
    assert_eq!(String::from(initials("ABZ")), "ABZ");
}

#[test]
fn initials_reject_anything_but_three_capital_letters() {
    for text in ["", "AB", "ABCD", "abc", "A1C", "A C", "ÀBC", "AÉ"] {
        assert_eq!(
            Initials::try_from(text.to_owned()),
            Err(InvalidInitials(text.to_owned())),
            "{text:?}"
        );
    }
}

#[test]
fn an_entry_starts_on_the_first_letter_of_aaa() {
    let entry = InitialsEntry::new();

    assert_eq!(entry.initials(), initials("AAA"));
    assert_eq!(entry.cursor(), 0);
    assert_eq!(InitialsEntry::default(), entry);
}

#[test]
fn up_and_down_step_the_letter_under_the_cursor() {
    let mut entry = InitialsEntry::new();
    tap(&mut entry, Dir::Up);
    tap(&mut entry, Dir::Up);
    assert_eq!(entry.initials(), initials("CAA"));

    tap(&mut entry, Dir::Down);
    assert_eq!(entry.initials(), initials("BAA"));
}

#[test]
fn letters_wrap_around_the_alphabet() {
    let mut entry = InitialsEntry::new();
    tap(&mut entry, Dir::Down);
    assert_eq!(entry.initials(), initials("ZAA"));

    tap(&mut entry, Dir::Up);
    assert_eq!(entry.initials(), initials("AAA"));
    for _ in 0..LETTER_COUNT {
        tap(&mut entry, Dir::Up);
    }
    assert_eq!(entry.initials(), initials("AAA"));
}

#[test]
fn left_and_right_move_the_cursor_within_the_initials() {
    let mut entry = InitialsEntry::new();
    tap(&mut entry, Dir::Left);
    assert_eq!(entry.cursor(), 0);

    for _ in 0..INITIALS_LEN + 1 {
        tap(&mut entry, Dir::Right);
    }
    assert_eq!(entry.cursor(), INITIALS_LEN - 1);
    tap(&mut entry, Dir::Up);
    assert_eq!(entry.initials(), initials("AAB"));

    tap(&mut entry, Dir::Left);
    assert_eq!(entry.cursor(), INITIALS_LEN - 2);
}

#[test]
fn confirm_moves_on_and_ends_on_the_last_letter() {
    let mut entry = InitialsEntry::new();
    tap(&mut entry, Dir::Up);
    assert_eq!(entry.tick(&CONFIRM), None);
    assert_eq!(entry.cursor(), 1);
    tap(&mut entry, Dir::Down);
    assert_eq!(entry.tick(&CONFIRM), None);
    tap(&mut entry, Dir::Up);
    tap(&mut entry, Dir::Up);

    assert_eq!(entry.tick(&CONFIRM), Some(initials("BZC")));
}

#[test]
fn a_direction_with_confirm_acts_before_it() {
    let mut entry = InitialsEntry::new();
    for _ in 1..INITIALS_LEN {
        entry.tick(&CONFIRM);
    }
    let input = InputState {
        confirm: true,
        ..held(Dir::Up)
    };

    assert_eq!(entry.tick(&input), Some(initials("AAB")));
}

/// The ticks, counted from the first, on which holding up steps the
/// letter, over the first [`REPEATS`] repeats.
fn count_steps() -> Vec<u32> {
    let mut entry = InitialsEntry::new();
    let ticks = INITIALS_REPEAT_DELAY_TICKS + REPEATS * INITIALS_REPEAT_TICKS;
    let mut steps = Vec::new();
    for tick in 0..=ticks {
        let before = entry.initials();
        entry.tick(&held(Dir::Up));
        if entry.initials() != before {
            steps.push(tick);
        }
    }
    steps
}

#[test]
fn a_held_direction_repeats_after_a_delay() {
    let repeats = (0..=REPEATS).map(|n| INITIALS_REPEAT_DELAY_TICKS + n * INITIALS_REPEAT_TICKS);
    let expected: Vec<_> = std::iter::once(0).chain(repeats).collect();

    assert_eq!(count_steps(), expected);
}

#[test]
fn changing_the_held_direction_acts_at_once() {
    let mut entry = InitialsEntry::new();
    entry.tick(&held(Dir::Up));
    entry.tick(&held(Dir::Right));
    entry.tick(&held(Dir::Up));

    assert_eq!(entry.cursor(), 1);
    assert_eq!(entry.initials(), initials("BBA"));
}
