use crate::tuning::{BOARD_H, BOARD_W, QIX_LENGTH, QIX_MAX_SPIN, QIX_SPEED, QIX_TRAIL_LEN};
use crate::{Board, Cell, DrawKind, Qix, Rng, Vec2};

/// Seed of the runs that need one fixed seed.
const SEED: u64 = 1193;

/// Ticks of the long runs.
const LONG_RUN_TICKS: u32 = 10_000;

/// Spacing of the points the free-cell check samples along a segment,
/// independently of the cell walk the Qix uses.
const SAMPLE_SPACING: f32 = 0.05;

/// Rounding slack on speeds and lengths.
const EPSILON: f32 = 1e-4;

/// A full-size board with claimed areas the Qix must bounce off: a block
/// in the middle and the bottom-left corner, as a claim leaves them.
fn board_with_claims() -> Board {
    let mut board = Board::new(BOARD_W, BOARD_H);
    let claimed = Cell::Claimed(DrawKind::Fast);
    for y in 0..BOARD_H {
        for x in 0..BOARD_W {
            let in_block = (60..100).contains(&x) && (40..80).contains(&y);
            let in_corner = x < 40 && y >= 80;
            if in_block || in_corner {
                board.set_cell(x, y, claimed);
            }
        }
    }
    board
}

/// A Qix lying along a free row of [`board_with_claims`], heading down
/// and right.
fn qix_beside_the_claims() -> Qix {
    let a = Vec2::new(10.5, 20.5);
    Qix::new(a, a + Vec2::new(QIX_LENGTH, 0.0), Vec2::new(0.4, 0.3), 0.0)
}

/// The cell of `board` under `point`, off the board reading as claimed.
fn board_cell(board: &Board, point: Vec2) -> Cell {
    board.cell(point.x.floor() as i32, point.y.floor() as i32)
}

/// Every sampled point of the Qix's segment, and the cell under its
/// midpoint, is a free cell of `board`.
fn assert_on_free_cells(board: &Board, qix: &Qix, tick: u32) {
    let (a, b) = (qix.a(), qix.b());
    let samples = ((b - a).length() / SAMPLE_SPACING).ceil() as u32;
    for i in 0..=samples {
        let point = a + (b - a) * (i as f32 / samples as f32);
        let cell = board_cell(board, point);
        assert_eq!(cell, Cell::Free, "tick {tick}: {point:?} of {qix:?}");
    }
    let (x, y) = qix.cell();
    let mid = board.cell(i32::from(x), i32::from(y));
    assert_eq!(mid, Cell::Free, "tick {tick}: midpoint of {qix:?}");
}

/// Ticks `qix` on `board` for `ticks` ticks, calling `check` after each.
fn run(board: &Board, qix: &mut Qix, ticks: u32, mut check: impl FnMut(&Qix, u32)) {
    let mut rng = Rng::new(SEED);
    for tick in 0..ticks {
        qix.tick(board, &mut rng);
        check(qix, tick);
    }
}

#[test]
fn stays_in_free_cells_of_a_fresh_board_over_ten_thousand_ticks() {
    let board = Board::new(BOARD_W, BOARD_H);
    let mut qix = Qix::spawn(&board, &mut Rng::new(SEED));

    run(&board, &mut qix, LONG_RUN_TICKS, |qix, tick| {
        assert_on_free_cells(&board, qix, tick);
    });
}

#[test]
fn stays_in_free_cells_around_claimed_areas_over_ten_thousand_ticks() {
    let board = board_with_claims();
    let mut qix = qix_beside_the_claims();

    run(&board, &mut qix, LONG_RUN_TICKS, |qix, tick| {
        assert_on_free_cells(&board, qix, tick);
    });
}

#[test]
fn keeps_its_length_speed_and_spin_within_bounds() {
    let board = board_with_claims();
    let mut qix = qix_beside_the_claims();

    run(&board, &mut qix, LONG_RUN_TICKS, |qix, tick| {
        let (length, speed) = ((qix.b() - qix.a()).length(), qix.vel().length());
        assert!(
            (length - QIX_LENGTH).abs() < EPSILON,
            "tick {tick}: {length}"
        );
        let bounds = QIX_SPEED.start() - EPSILON..=QIX_SPEED.end() + EPSILON;
        assert!(bounds.contains(&speed), "tick {tick}: speed {speed}");
        assert!(
            qix.spin().abs() <= QIX_MAX_SPIN,
            "tick {tick}: {}",
            qix.spin()
        );
    });
}

#[test]
fn the_trail_holds_the_last_segments_up_to_its_length() {
    let board = board_with_claims();
    let mut qix = qix_beside_the_claims();
    let mut segments = vec![(qix.a(), qix.b())];
    assert_eq!(qix.trail().len(), 1);

    run(&board, &mut qix, LONG_RUN_TICKS, |qix, _| {
        segments.push((qix.a(), qix.b()));
        let expected = &segments[segments.len().saturating_sub(QIX_TRAIL_LEN)..];
        assert!(qix.trail().iter().eq(expected.iter()));
    });
    assert_eq!(qix.trail().len(), QIX_TRAIL_LEN);
}

#[test]
fn a_seed_fixes_the_path() {
    let board = Board::new(BOARD_W, BOARD_H);
    let path = |seed| {
        let mut rng = Rng::new(seed);
        let mut qix = Qix::spawn(&board, &mut rng);
        (0..LONG_RUN_TICKS).for_each(|_| qix.tick(&board, &mut rng));
        qix
    };

    assert_eq!(path(SEED), path(SEED));
    assert_ne!(path(SEED), path(SEED + 1));
}

#[test]
fn bounces_off_the_edge_of_the_board() {
    let board = Board::new(BOARD_W, BOARD_H);
    let x = f32::from(BOARD_W) - 0.5;
    let a = Vec2::new(x, 10.5);
    let speed = *QIX_SPEED.end();
    let mut qix = Qix::new(
        a,
        a + Vec2::new(0.0, QIX_LENGTH),
        Vec2::new(speed, 0.0),
        0.0,
    );

    qix.tick(&board, &mut Rng::new(SEED));

    assert!(qix.vel().x < 0.0, "{qix:?}");
    assert_on_free_cells(&board, &qix, 0);
}

#[test]
fn bounces_off_a_claimed_cell() {
    let mut board = Board::new(BOARD_W, BOARD_H);
    let wall = 30;
    for y in 0..BOARD_H {
        board.set_cell(wall, y, Cell::Claimed(DrawKind::Slow));
    }
    let a = Vec2::new(f32::from(wall) - 0.5, 10.5);
    let speed = *QIX_SPEED.end();
    let mut qix = Qix::new(
        a,
        a + Vec2::new(0.0, QIX_LENGTH),
        Vec2::new(speed, 0.0),
        0.0,
    );

    qix.tick(&board, &mut Rng::new(SEED));

    assert!(qix.vel().x < 0.0, "{qix:?}");
    assert_on_free_cells(&board, &qix, 0);
}

#[test]
fn spawns_centred_on_the_board_within_the_speed_bounds() {
    let board = Board::new(BOARD_W, BOARD_H);

    let qix = Qix::spawn(&board, &mut Rng::new(SEED));

    let centre = Vec2::new(f32::from(BOARD_W), f32::from(BOARD_H)) * 0.5;
    assert!((qix.a().midpoint(qix.b()) - centre).length() < EPSILON);
    assert!(((qix.b() - qix.a()).length() - QIX_LENGTH).abs() < EPSILON);
    assert!(QIX_SPEED.contains(&qix.vel().length()), "{qix:?}");
    assert_eq!(qix.trail().len(), 1);
}

#[test]
fn its_cell_is_the_cell_under_its_midpoint() {
    let qix = Qix::new(
        Vec2::new(2.5, 3.2),
        Vec2::new(6.9, 3.6),
        Vec2::default(),
        0.0,
    );

    assert_eq!(qix.cell(), (4, 3));
}
