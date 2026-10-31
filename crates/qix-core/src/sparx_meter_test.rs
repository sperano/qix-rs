use crate::{MeterExpiry, SparxMeter};

/// Ticks of one countdown of the meters the tests run.
const LENGTH: u32 = 5;

/// Ticks `meter` `ticks` times and returns the tick, from one, of each
/// expiry with the expiry.
fn expiries(meter: &mut SparxMeter, ticks: u32) -> Vec<(u32, MeterExpiry)> {
    (1..=ticks)
        .filter_map(|tick| meter.tick().map(|expiry| (tick, expiry)))
        .collect()
}

#[test]
fn a_new_meter_is_full_and_counts_to_the_release() {
    let meter = SparxMeter::new(LENGTH);

    assert_eq!(meter.length(), LENGTH);
    assert_eq!(meter.remaining(), LENGTH);
    assert_eq!(meter.next_expiry(), Some(MeterExpiry::Release));
}

#[test]
fn each_tick_counts_one_down() {
    let mut meter = SparxMeter::new(LENGTH);

    assert_eq!(meter.tick(), None);
    assert_eq!(meter.tick(), None);

    assert_eq!(meter.remaining(), LENGTH - 2);
}

#[test]
fn the_meter_releases_then_turns_super_one_length_apart() {
    let mut meter = SparxMeter::new(LENGTH);

    let expiries = expiries(&mut meter, 3 * LENGTH);

    assert_eq!(
        expiries,
        [
            (LENGTH, MeterExpiry::Release),
            (2 * LENGTH, MeterExpiry::Super)
        ]
    );
}

#[test]
fn the_first_expiry_starts_the_second_countdown_full() {
    let mut meter = SparxMeter::new(LENGTH);

    expiries(&mut meter, LENGTH);

    assert_eq!(meter.remaining(), LENGTH);
    assert_eq!(meter.next_expiry(), Some(MeterExpiry::Super));
}

#[test]
fn a_spent_meter_stays_empty() {
    let mut meter = SparxMeter::new(LENGTH);
    expiries(&mut meter, 2 * LENGTH);

    assert_eq!(meter.tick(), None);

    assert_eq!(meter.remaining(), 0);
    assert_eq!(meter.next_expiry(), None);
}

#[test]
fn reset_fills_a_running_meter() {
    let mut meter = SparxMeter::new(LENGTH);
    expiries(&mut meter, LENGTH + 1);

    meter.reset();

    assert_eq!(meter, SparxMeter::new(LENGTH));
}

#[test]
fn a_reset_spent_meter_expires_both_times_again() {
    let mut meter = SparxMeter::new(LENGTH);
    expiries(&mut meter, 2 * LENGTH);

    meter.reset();

    assert_eq!(
        expiries(&mut meter, 2 * LENGTH),
        [
            (LENGTH, MeterExpiry::Release),
            (2 * LENGTH, MeterExpiry::Super)
        ]
    );
}

#[test]
fn a_zero_length_meter_counts_one_tick() {
    let mut meter = SparxMeter::new(0);

    assert_eq!(meter.length(), 1);
    assert_eq!(meter.tick(), Some(MeterExpiry::Release));
    assert_eq!(meter.tick(), Some(MeterExpiry::Super));
}
