use crate::score::{Score, score_for_claim};
use crate::tuning::{POINTS_PER_CELL, SLOW_DRAW_FACTOR};
use crate::{ClaimResult, DrawKind};

/// Cells of the claims the tests score.
const CELLS: usize = 37;

/// A level multiplier above one.
const MULTIPLIER: u32 = 3;

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
