use crate::tuning::{MAX_LIVES, STARTING_LIVES, STARTING_LIVES_RANGE};
use crate::{ClaimThreshold, Rules, StartingLives};

#[test]
fn starting_lives_in_the_range_are_kept() {
    for lives in STARTING_LIVES_RANGE {
        assert_eq!(StartingLives::new(lives).get(), lives);
    }
}

#[test]
fn starting_lives_outside_the_range_are_clamped_into_it() {
    let (fewest, most) = (*STARTING_LIVES_RANGE.start(), *STARTING_LIVES_RANGE.end());

    assert_eq!(StartingLives::new(0).get(), fewest);
    assert_eq!(StartingLives::new(most + 1).get(), most);
    assert_eq!(StartingLives::new(u32::MAX).get(), most);
}

#[test]
fn a_game_may_start_with_up_to_the_most_lives() {
    assert_eq!(*STARTING_LIVES_RANGE.end(), MAX_LIVES);
}

#[test]
fn starting_lives_convert_through_their_number_clamped() {
    assert_eq!(
        u32::from(StartingLives::from(STARTING_LIVES)),
        STARTING_LIVES
    );
    assert_eq!(StartingLives::from(0), StartingLives::new(0));
}

#[test]
fn the_default_rules_are_the_arcade_ones() {
    let rules = Rules::default();

    assert_eq!(rules.threshold, ClaimThreshold::default());
    assert_eq!(rules.starting_lives.get(), STARTING_LIVES);
}
