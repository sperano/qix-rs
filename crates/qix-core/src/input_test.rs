use crate::tuning::{REPEAT_DELAY_TICKS, REPEAT_TICKS};
use crate::{Dir, DirRepeat, DrawKind, InputState};

/// Repeats [`ticks_acting`] holds a direction for.
const REPEATS: u32 = 3;

fn buttons(fast: bool, slow: bool) -> InputState {
    InputState {
        fast,
        slow,
        ..InputState::default()
    }
}

#[test]
fn no_draw_button_draws_nothing() {
    assert_eq!(buttons(false, false).draw_kind(), None);
}

#[test]
fn each_draw_button_draws_its_kind() {
    assert_eq!(buttons(true, false).draw_kind(), Some(DrawKind::Fast));
    assert_eq!(buttons(false, true).draw_kind(), Some(DrawKind::Slow));
}

#[test]
fn fast_wins_when_both_draw_buttons_are_held() {
    assert_eq!(buttons(true, true).draw_kind(), Some(DrawKind::Fast));
}

/// The ticks, counted from zero, on which holding `dir` for `ticks`
/// ticks acts.
fn ticks_acting(dir: Option<Dir>, ticks: u32) -> Vec<u32> {
    let mut repeat = DirRepeat::new();
    (0..ticks).filter(|_| repeat.tick(dir).is_some()).collect()
}

#[test]
fn a_held_direction_acts_at_once_then_repeats_after_the_delay() {
    let ticks = REPEAT_DELAY_TICKS + REPEATS * REPEAT_TICKS + 1;

    let acting = ticks_acting(Some(Dir::Up), ticks);

    let repeats = (0..=REPEATS).map(|n| REPEAT_DELAY_TICKS + n * REPEAT_TICKS);
    assert_eq!(
        acting,
        std::iter::once(0).chain(repeats).collect::<Vec<_>>()
    );
}

#[test]
fn no_direction_never_acts() {
    assert_eq!(
        ticks_acting(None, REPEAT_DELAY_TICKS * 2),
        Vec::<u32>::new()
    );
}

#[test]
fn a_new_direction_acts_at_once() {
    let mut repeat = DirRepeat::new();
    repeat.tick(Some(Dir::Up));

    assert_eq!(repeat.tick(Some(Dir::Left)), Some(Dir::Left));
    assert_eq!(repeat.tick(Some(Dir::Left)), None);
}

#[test]
fn a_released_direction_acts_at_once_when_held_again() {
    let mut repeat = DirRepeat::new();
    repeat.tick(Some(Dir::Up));
    repeat.tick(None);

    assert_eq!(repeat.tick(Some(Dir::Up)), Some(Dir::Up));
}
