use crate::ClaimThreshold;
use crate::tuning::{
    BOARD_H, BOARD_W, BONUS_PER_PERCENT, CLAIM_THRESHOLD_DEFAULT, CLAIM_THRESHOLD_RANGE,
};

/// Cells of a full-size board.
const FULL_BOARD_CELLS: u32 = BOARD_W as u32 * BOARD_H as u32;

/// Whole percent points in a board.
const WHOLE: u32 = 100;

#[test]
fn the_default_threshold_is_the_tuned_default() {
    assert_eq!(
        ClaimThreshold::default().fraction(),
        CLAIM_THRESHOLD_DEFAULT
    );
}

#[test]
fn a_threshold_in_range_is_kept() {
    assert_eq!(ClaimThreshold::new(0.6).fraction(), 0.6);
}

#[test]
fn a_threshold_out_of_range_is_clamped_into_it() {
    let (low, high) = (*CLAIM_THRESHOLD_RANGE.start(), *CLAIM_THRESHOLD_RANGE.end());

    assert_eq!(ClaimThreshold::new(0.1).fraction(), low);
    assert_eq!(ClaimThreshold::new(1.0).fraction(), high);
    assert_eq!(ClaimThreshold::new(f32::NEG_INFINITY).fraction(), low);
    assert_eq!(ClaimThreshold::new(f32::INFINITY).fraction(), high);
}

#[test]
fn a_nan_threshold_is_the_default() {
    assert_eq!(ClaimThreshold::new(f32::NAN), ClaimThreshold::default());
}

#[test]
fn exactly_the_threshold_reaches_it() {
    let threshold = ClaimThreshold::default();

    assert!(threshold.is_reached(CLAIM_THRESHOLD_DEFAULT));
    assert!(threshold.is_reached(1.0));
    assert!(!threshold.is_reached(CLAIM_THRESHOLD_DEFAULT.next_down()));
}

#[test]
fn exactly_the_threshold_scores_no_bonus() {
    assert_eq!(ClaimThreshold::default().bonus(CLAIM_THRESHOLD_DEFAULT), 0);
}

#[test]
fn below_the_threshold_scores_no_bonus() {
    assert_eq!(ClaimThreshold::default().bonus(0.0), 0);
}

#[test]
fn every_whole_percent_above_the_threshold_scores_the_bonus_per_percent() {
    let threshold = ClaimThreshold::default();
    let at = (CLAIM_THRESHOLD_DEFAULT * WHOLE as f32) as u32;

    for percent in at..=WHOLE {
        let fraction = percent as f32 / WHOLE as f32;
        let bonus = (percent - at) * BONUS_PER_PERCENT;
        assert_eq!(threshold.bonus(fraction), bonus, "{percent} %");
    }
}

#[test]
fn a_part_of_a_percent_scores_nothing() {
    let threshold = ClaimThreshold::default();

    assert_eq!(threshold.bonus(0.759), 0);
    assert_eq!(threshold.bonus(0.789), 3 * BONUS_PER_PERCENT);
}

#[test]
fn the_bonus_counts_from_a_threshold_set_lower() {
    let threshold = ClaimThreshold::new(*CLAIM_THRESHOLD_RANGE.start());

    assert_eq!(threshold.bonus(1.0), 50 * BONUS_PER_PERCENT);
}

#[test]
fn a_cell_short_of_a_whole_percent_on_a_full_board_does_not_reach_it() {
    let threshold = ClaimThreshold::default();
    let fraction = |cells: u32| cells as f32 / FULL_BOARD_CELLS as f32;
    // One percent point above the default threshold.
    let cells = FULL_BOARD_CELLS / WHOLE * 76;

    assert_eq!(threshold.bonus(fraction(cells)), BONUS_PER_PERCENT);
    assert_eq!(threshold.bonus(fraction(cells - 1)), 0);
}
