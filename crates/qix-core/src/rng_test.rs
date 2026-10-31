use crate::Rng;

/// Values drawn by the sequence tests.
const DRAWS: usize = 1000;

#[test]
fn matches_the_splitmix64_reference_output() {
    // First outputs of the reference SplitMix64 seeded with 0.
    let mut rng = Rng::new(0);

    assert_eq!(rng.next_u64(), 0xE220_A839_7B1D_CDAF);
    assert_eq!(rng.next_u64(), 0x6E78_9E6A_A1B9_65F4);
}

#[test]
fn a_seed_fixes_the_sequence() {
    let (mut first, mut second) = (Rng::new(42), Rng::new(42));

    for _ in 0..DRAWS {
        assert_eq!(first.next_u64(), second.next_u64());
    }
}

#[test]
fn different_seeds_give_different_sequences() {
    let (mut first, mut second) = (Rng::new(1), Rng::new(2));

    let same = (0..DRAWS).all(|_| first.next_u64() == second.next_u64());

    assert!(!same);
}

#[test]
fn fractions_lie_in_the_unit_interval() {
    let mut rng = Rng::new(7);

    for _ in 0..DRAWS {
        let value = rng.next_f32();
        assert!((0.0..1.0).contains(&value), "{value}");
    }
}

#[test]
fn ranges_stay_within_their_bounds() {
    let mut rng = Rng::new(7);

    for _ in 0..DRAWS {
        let value = rng.range_f32(-0.5, 2.0);
        assert!((-0.5..2.0).contains(&value), "{value}");
    }
}

#[test]
fn an_empty_range_gives_its_bound() {
    let mut rng = Rng::new(7);

    assert_eq!(rng.range_f32(3.0, 3.0), 3.0);
}
