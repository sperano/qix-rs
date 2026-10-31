use crate::tuning::{
    BOARD_H, BOARD_W, BONUS_PER_PERCENT, CLAIM_THRESHOLD_DEFAULT, CLAIM_THRESHOLD_RANGE,
    DIFFICULTY_RAMP_LEVELS, FIRST_LEVEL, LEVEL_SPARX, QIX_SPEED, SPARX_METER_TICKS,
    TWO_QIX_FROM_LEVEL,
};
use crate::{ClaimThreshold, LevelConfig};

/// Cells of a full-size board.
const FULL_BOARD_CELLS: u32 = BOARD_W as u32 * BOARD_H as u32;

/// The last level of the difficulty ramp.
const RAMP_END: u32 = FIRST_LEVEL + DIFFICULTY_RAMP_LEVELS;

/// The last level the difficulty tests walk to: well past the ramp.
const LAST_LEVEL_TESTED: u32 = RAMP_END + DIFFICULTY_RAMP_LEVELS;

/// Qix of a level before [`TWO_QIX_FROM_LEVEL`], and from it on.
const ONE_QIX: usize = 1;
const TWO_QIX: usize = 2;

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

#[test]
fn the_first_level_has_the_first_level_tuning() {
    assert_eq!(
        LevelConfig::for_level(FIRST_LEVEL),
        LevelConfig {
            qix_speed: QIX_SPEED,
            qix_count: ONE_QIX,
            sparx_count: LEVEL_SPARX,
            meter_ticks: SPARX_METER_TICKS,
        }
    );
}

#[test]
fn a_level_below_the_first_plays_as_the_first() {
    assert_eq!(
        LevelConfig::for_level(FIRST_LEVEL - 1),
        LevelConfig::for_level(FIRST_LEVEL)
    );
}

#[test]
fn difficulty_never_drops_from_one_level_to_the_next() {
    for level in FIRST_LEVEL..LAST_LEVEL_TESTED {
        let (this, next) = (
            LevelConfig::for_level(level),
            LevelConfig::for_level(level + 1),
        );
        assert!(
            next.qix_speed.start() >= this.qix_speed.start(),
            "level {level}"
        );
        assert!(
            next.qix_speed.end() >= this.qix_speed.end(),
            "level {level}"
        );
        assert!(next.qix_count >= this.qix_count, "level {level}");
        assert!(next.sparx_count >= this.sparx_count, "level {level}");
        assert!(next.meter_ticks <= this.meter_ticks, "level {level}");
    }
}

#[test]
fn each_level_of_the_ramp_has_faster_qix_and_a_shorter_meter() {
    for level in FIRST_LEVEL..RAMP_END {
        let (this, next) = (
            LevelConfig::for_level(level),
            LevelConfig::for_level(level + 1),
        );
        assert!(
            next.qix_speed.start() > this.qix_speed.start(),
            "level {level}"
        );
        assert!(next.qix_speed.end() > this.qix_speed.end(), "level {level}");
        assert!(next.meter_ticks < this.meter_ticks, "level {level}");
    }
}

#[test]
fn the_ramp_adds_sparx() {
    assert!(
        LevelConfig::for_level(RAMP_END).sparx_count
            > LevelConfig::for_level(FIRST_LEVEL).sparx_count
    );
}

#[test]
fn difficulty_stops_rising_after_the_ramp() {
    let top = LevelConfig::for_level(RAMP_END);

    assert!(top.meter_ticks > 0);
    for level in [RAMP_END + 1, LAST_LEVEL_TESTED, u32::MAX] {
        assert_eq!(LevelConfig::for_level(level), top, "level {level}");
    }
}

#[test]
fn there_are_two_qix_from_level_three_on() {
    assert_eq!(TWO_QIX_FROM_LEVEL, 3);
    for level in FIRST_LEVEL..TWO_QIX_FROM_LEVEL {
        assert_eq!(
            LevelConfig::for_level(level).qix_count,
            ONE_QIX,
            "level {level}"
        );
    }
    for level in TWO_QIX_FROM_LEVEL..=LAST_LEVEL_TESTED {
        assert_eq!(
            LevelConfig::for_level(level).qix_count,
            TWO_QIX,
            "level {level}"
        );
    }
}
