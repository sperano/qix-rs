use crate::{DrawKind, InputState};

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
