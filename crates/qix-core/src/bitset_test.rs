use crate::BitSet;

/// A capacity spanning more than one word, ending inside the last one.
const CAPACITY: usize = 130;

#[test]
fn a_new_set_is_empty() {
    let set = BitSet::new(CAPACITY);

    assert_eq!(set.capacity(), CAPACITY);
    assert_eq!(set.count(), 0);
    assert!((0..CAPACITY).all(|index| !set.contains(index)));
}

#[test]
fn inserted_indices_are_contained() {
    let mut set = BitSet::new(CAPACITY);

    for index in [0, 63, 64, CAPACITY - 1] {
        assert!(set.insert(index));
    }

    assert_eq!(set.count(), 4);
    assert!(set.contains(0) && set.contains(63));
    assert!(set.contains(64) && set.contains(CAPACITY - 1));
    assert!(!set.contains(1) && !set.contains(65));
}

#[test]
fn inserting_twice_reports_the_index_was_there() {
    let mut set = BitSet::new(CAPACITY);

    assert!(set.insert(7));
    assert!(!set.insert(7));
    assert_eq!(set.count(), 1);
}

#[test]
fn an_index_past_the_capacity_is_never_contained() {
    let set = BitSet::new(CAPACITY);

    assert!(!set.contains(CAPACITY));
    assert!(!set.contains(usize::MAX));
}

#[test]
#[should_panic(expected = "past the capacity")]
fn inserting_past_the_capacity_panics() {
    BitSet::new(CAPACITY).insert(CAPACITY);
}
