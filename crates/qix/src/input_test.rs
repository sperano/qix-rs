use eframe::egui::{self, Event, Key, Modifiers};
use qix_core::{Dir, InputState};

use crate::input::{InputLatch, overlay_toggled, sample};

fn holding(keys: &[Key]) -> egui::InputState {
    let mut input = egui::InputState::default();
    input.keys_down.extend(keys);
    input
}

fn key_event(key: Key, pressed: bool, repeat: bool) -> Event {
    Event::Key {
        key,
        physical_key: None,
        pressed,
        repeat,
        modifiers: Modifiers::NONE,
    }
}

fn with_events(events: Vec<Event>) -> egui::InputState {
    let mut input = egui::InputState::default();
    input.events = events;
    input
}

fn pressing_confirm() -> InputState {
    InputState {
        confirm: true,
        ..InputState::default()
    }
}

#[test]
fn no_key_is_no_input() {
    assert_eq!(sample(&holding(&[])), InputState::default());
}

#[test]
fn held_keys_set_direction_and_draw_speed() {
    let input = sample(&holding(&[Key::ArrowLeft, Key::Z, Key::X]));
    assert_eq!(
        input,
        InputState {
            dir: Some(Dir::Left),
            fast: true,
            slow: true,
            ..InputState::default()
        }
    );
}

#[test]
fn each_arrow_maps_to_its_direction() {
    let arrows = [
        (Key::ArrowUp, Dir::Up),
        (Key::ArrowDown, Dir::Down),
        (Key::ArrowLeft, Dir::Left),
        (Key::ArrowRight, Dir::Right),
    ];
    for (key, dir) in arrows {
        assert_eq!(sample(&holding(&[key])).dir, Some(dir), "{key:?}");
    }
}

#[test]
fn the_first_direction_in_priority_order_wins() {
    let input = sample(&holding(&[Key::ArrowRight, Key::ArrowDown]));
    assert_eq!(input.dir, Some(Dir::Down));
}

#[test]
fn fresh_presses_set_confirm_and_pause() {
    let input = sample(&with_events(vec![
        key_event(Key::Enter, true, false),
        key_event(Key::Escape, true, false),
    ]));
    assert!(input.confirm);
    assert!(input.pause);
}

#[test]
fn key_repeats_and_releases_are_not_presses() {
    let input = sample(&with_events(vec![
        key_event(Key::Enter, true, true),
        key_event(Key::Escape, false, false),
    ]));
    assert!(!input.confirm);
    assert!(!input.pause);
}

#[test]
fn a_press_in_a_frame_without_ticks_reaches_the_next_tick() {
    let mut latch = InputLatch::default();
    latch.record(pressing_confirm());
    latch.record(InputState::default());
    assert!(latch.next_tick().confirm);
}

#[test]
fn a_press_reaches_only_the_first_tick_of_a_frame() {
    let mut latch = InputLatch::default();
    latch.record(pressing_confirm());
    assert!(latch.next_tick().confirm);
    assert!(!latch.next_tick().confirm);
}

#[test]
fn held_controls_follow_the_latest_frame_on_every_tick() {
    let mut latch = InputLatch::default();
    let held = InputState {
        dir: Some(Dir::Up),
        fast: true,
        ..InputState::default()
    };
    latch.record(held);
    assert_eq!(latch.next_tick(), held);
    assert_eq!(latch.next_tick(), held);

    latch.record(InputState::default());
    assert_eq!(latch.next_tick(), InputState::default());
}

#[test]
fn a_fresh_f1_press_toggles_the_overlay() {
    assert!(overlay_toggled(&with_events(vec![key_event(
        Key::F1,
        true,
        false
    )])));
}

#[test]
fn a_held_or_released_f1_does_not_toggle_the_overlay() {
    let events = vec![
        key_event(Key::F1, true, true),
        key_event(Key::F1, false, false),
    ];
    assert!(!overlay_toggled(&with_events(events)));
    assert!(!overlay_toggled(&holding(&[Key::F1])));
}

#[test]
fn the_overlay_key_is_not_game_input() {
    let input = with_events(vec![key_event(Key::F1, true, false)]);
    assert_eq!(sample(&input), InputState::default());
}
