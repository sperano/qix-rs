use crate::tuning::ATTRACT_DELAY_TICKS;
use crate::{Attract, DEMO_SCRIPT, Dir, InputState};

/// Ticks `attract` `ticks` times with nothing held.
fn idle(attract: &mut Attract, ticks: u32) {
    for _ in 0..ticks {
        attract.tick(&InputState::default());
    }
}

fn script_ticks() -> u32 {
    DEMO_SCRIPT.iter().map(|step| step.ticks).sum()
}

fn pressed() -> InputState {
    InputState {
        dir: Some(Dir::Left),
        ..InputState::default()
    }
}

#[test]
fn a_new_attract_mode_shows_no_demo() {
    assert!(Attract::new().demo().is_none());
}

#[test]
fn the_demo_starts_after_the_delay_of_idle_ticks() {
    let mut attract = Attract::new();

    idle(&mut attract, ATTRACT_DELAY_TICKS - 1);
    assert!(attract.demo().is_none());
    idle(&mut attract, 1);

    let demo = attract.demo().expect("a demo");
    assert_eq!(demo.game().ticks(), 0);
}

#[test]
fn a_running_demo_ticks_with_each_idle_tick() {
    let mut attract = Attract::new();
    idle(&mut attract, ATTRACT_DELAY_TICKS);

    idle(&mut attract, 3);

    assert_eq!(attract.demo().expect("a demo").game().ticks(), 3);
}

#[test]
fn input_before_the_delay_restarts_the_count() {
    let mut attract = Attract::new();
    idle(&mut attract, ATTRACT_DELAY_TICKS - 1);

    attract.tick(&pressed());
    idle(&mut attract, ATTRACT_DELAY_TICKS - 1);

    assert!(attract.demo().is_none());
    idle(&mut attract, 1);
    assert!(attract.demo().is_some());
}

#[test]
fn input_stops_the_demo_and_restarts_the_count() {
    let mut attract = Attract::new();
    idle(&mut attract, ATTRACT_DELAY_TICKS + 1);
    assert!(attract.demo().is_some());

    attract.tick(&pressed());

    assert!(attract.demo().is_none());
    idle(&mut attract, ATTRACT_DELAY_TICKS - 1);
    assert!(attract.demo().is_none());
    idle(&mut attract, 1);
    assert!(attract.demo().is_some());
}

#[test]
fn a_press_of_confirm_counts_as_input() {
    let mut attract = Attract::new();
    idle(&mut attract, ATTRACT_DELAY_TICKS);

    attract.tick(&InputState {
        confirm: true,
        ..InputState::default()
    });

    assert!(attract.demo().is_none());
}

#[test]
fn a_finished_demo_goes_back_to_idle_and_starts_again_after_the_delay() {
    let mut attract = Attract::new();
    idle(&mut attract, ATTRACT_DELAY_TICKS);

    idle(&mut attract, script_ticks() - 1);
    assert!(attract.demo().is_some());
    idle(&mut attract, 1);
    assert!(attract.demo().is_none());

    idle(&mut attract, ATTRACT_DELAY_TICKS - 1);
    assert!(attract.demo().is_none());
    idle(&mut attract, 1);
    let demo = attract.demo().expect("a second demo");
    assert_eq!(demo.game().ticks(), 0);
}

#[test]
fn reset_drops_the_demo_and_restarts_the_count() {
    let mut attract = Attract::new();
    idle(&mut attract, ATTRACT_DELAY_TICKS);

    attract.reset();

    assert!(attract.demo().is_none());
    idle(&mut attract, ATTRACT_DELAY_TICKS - 1);
    assert!(attract.demo().is_none());
    idle(&mut attract, 1);
    assert!(attract.demo().is_some());
}

#[test]
fn reset_before_the_delay_restarts_the_count() {
    let mut attract = Attract::new();
    idle(&mut attract, ATTRACT_DELAY_TICKS - 1);

    attract.reset();
    idle(&mut attract, 1);

    assert!(attract.demo().is_none());
}
