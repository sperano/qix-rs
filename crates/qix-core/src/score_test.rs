use crate::score::{Score, extra_lives, score_for_claim};
use crate::tuning::{
    EXTRA_LIFE_EVERY, MAX_LIVES, POINTS_PER_CELL, SLOW_DRAW_FACTOR, STARTING_LIVES,
};
use crate::{ClaimResult, DrawKind};

/// Cells of the claims the tests score.
const CELLS: usize = 37;

/// A level multiplier above one.
const MULTIPLIER: u32 = 3;

/// Steps of [`EXTRA_LIFE_EVERY`] the several-steps tests cross at once.
const STEPS: u32 = 3;

fn score(points: u32) -> Score {
    let mut score = Score::default();
    score.add(points);
    score
}

fn claim(cells: usize, kind: DrawKind) -> ClaimResult {
    ClaimResult {
        cells,
        fraction: 0.5,
        kind,
    }
}

#[test]
fn a_fast_claim_scores_its_cells_at_points_per_cell() {
    let points = score_for_claim(&claim(CELLS, DrawKind::Fast), 1);

    assert_eq!(points, CELLS as u32 * POINTS_PER_CELL);
}

#[test]
fn the_level_multiplier_scales_the_points() {
    let points = score_for_claim(&claim(CELLS, DrawKind::Fast), MULTIPLIER);

    assert_eq!(points, CELLS as u32 * POINTS_PER_CELL * MULTIPLIER);
}

#[test]
fn a_slow_claim_scores_the_slow_draw_factor_times_a_fast_one() {
    let fast = score_for_claim(&claim(CELLS, DrawKind::Fast), MULTIPLIER);
    let slow = score_for_claim(&claim(CELLS, DrawKind::Slow), MULTIPLIER);

    assert_eq!(slow, fast * SLOW_DRAW_FACTOR);
}

#[test]
fn a_claim_too_large_to_count_saturates() {
    let points = score_for_claim(&claim(usize::MAX, DrawKind::Slow), MULTIPLIER);

    assert_eq!(points, u32::MAX);
}

#[test]
fn a_new_score_is_zero() {
    assert_eq!(Score::default().points(), 0);
}

#[test]
fn add_accumulates_points() {
    let mut score = Score::default();

    score.add(5);
    score.add(7);

    assert_eq!(score.points(), 12);
}

#[test]
fn add_saturates_at_the_maximum() {
    let mut score = Score::default();

    score.add(u32::MAX - 1);
    score.add(2);

    assert_eq!(score.points(), u32::MAX);
}

#[test]
fn points_below_the_next_step_earn_no_life() {
    let lives = extra_lives(score(1), score(EXTRA_LIFE_EVERY - 1), STARTING_LIVES);

    assert_eq!(lives, 0);
}

#[test]
fn reaching_a_step_exactly_earns_a_life() {
    let lives = extra_lives(
        score(EXTRA_LIFE_EVERY - 1),
        score(EXTRA_LIFE_EVERY),
        STARTING_LIVES,
    );

    assert_eq!(lives, 1);
}

#[test]
fn points_from_a_step_to_below_the_next_earn_no_life() {
    let before = score(EXTRA_LIFE_EVERY);

    let lives = extra_lives(before, score(2 * EXTRA_LIFE_EVERY - 1), STARTING_LIVES);

    assert_eq!(lives, 0);
}

#[test]
fn crossing_several_steps_at_once_earns_a_life_for_each() {
    let after = score(STEPS * EXTRA_LIFE_EVERY + 1);

    let lives = extra_lives(score(EXTRA_LIFE_EVERY - 1), after, STARTING_LIVES);

    assert_eq!(lives, STEPS);
}

#[test]
fn extra_lives_stop_at_the_maximum() {
    let after = score(STEPS * EXTRA_LIFE_EVERY);

    let lives = extra_lives(Score::default(), after, MAX_LIVES - 1);

    assert_eq!(lives, 1);
}

#[test]
fn a_player_at_the_maximum_earns_no_life() {
    let after = score(STEPS * EXTRA_LIFE_EVERY);

    assert_eq!(extra_lives(Score::default(), after, MAX_LIVES), 0);
    assert_eq!(extra_lives(Score::default(), after, MAX_LIVES + 1), 0);
}

#[test]
fn a_saturated_score_earns_no_more_lives() {
    let lives = extra_lives(score(u32::MAX), score(u32::MAX), 1);

    assert_eq!(lives, 0);
}
