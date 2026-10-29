use qix_core::tuning::TICK_HZ;

use crate::timestep::{FixedTimestep, MAX_TICKS_PER_FRAME};

/// Frame times to check, in seconds: refresh rates above, at and below
/// the tick rate, and times that do not divide a tick evenly.
const FRAME_TIMES: [f64; 7] = [
    1.0 / 144.0,
    1.0 / 120.0,
    1.0 / 75.0,
    1.0 / 60.0,
    1.0 / 30.0,
    1.0 / 64.0,
    0.01,
];

/// Frames run for each frame time.
const FRAMES: u32 = 10_000;

fn ticks_for(timestep: &mut FixedTimestep, dt: f32, frames: u32) -> u32 {
    (0..frames).map(|_| timestep.advance(dt)).sum()
}

fn tick_period() -> f32 {
    1.0 / TICK_HZ as f32
}

#[test]
fn n_frames_of_dt_run_n_dt_tick_hz_ticks() {
    for dt in FRAME_TIMES {
        let expected = (f64::from(FRAMES) * dt * f64::from(TICK_HZ)).floor() as u32;
        let ticks = ticks_for(&mut FixedTimestep::default(), dt as f32, FRAMES);
        assert_eq!(ticks, expected, "dt = {dt}");
    }
}

#[test]
fn a_frame_of_one_tick_period_runs_exactly_one_tick() {
    let mut timestep = FixedTimestep::default();
    for frame in 0..FRAMES {
        assert_eq!(timestep.advance(tick_period()), 1, "frame {frame}");
    }
}

#[test]
fn a_frame_shorter_than_a_tick_carries_its_time_over() {
    let mut timestep = FixedTimestep::default();
    let half = tick_period() / 2.0;
    assert_eq!(timestep.advance(half), 0);
    assert_eq!(timestep.advance(half), 1);
}

#[test]
fn a_stall_is_capped_and_its_backlog_dropped() {
    let mut timestep = FixedTimestep::default();
    assert_eq!(timestep.advance(1.0), MAX_TICKS_PER_FRAME);
    assert_eq!(timestep.advance(tick_period()), 1);
}

#[test]
fn invalid_frame_times_run_no_tick_and_leave_the_clock_intact() {
    let mut timestep = FixedTimestep::default();
    assert_eq!(timestep.advance(f32::NAN), 0);
    assert_eq!(timestep.advance(-1.0), 0);
    assert_eq!(timestep.advance(tick_period()), 1);
}
