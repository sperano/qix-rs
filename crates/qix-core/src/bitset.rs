//! A fixed-size set of indices, one bit per index.

/// Bits in one word of a [`BitSet`].
const WORD_BITS: usize = u64::BITS as usize;

/// A set of the indices `0..capacity`, stored as one bit each.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BitSet {
    capacity: usize,
    words: Vec<u64>,
}

impl BitSet {
    /// An empty set that can hold the indices `0..capacity`.
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            words: vec![0; capacity.div_ceil(WORD_BITS)],
        }
    }

    /// The number of indices the set can hold: it holds `0..capacity()`.
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// `index` is in the set. An index past the capacity never is.
    pub fn contains(&self, index: usize) -> bool {
        index < self.capacity && self.words[index / WORD_BITS] & bit(index) != 0
    }

    /// Adds `index` to the set; `true` when it was not in it yet.
    ///
    /// # Panics
    ///
    /// If `index` is not below the capacity.
    pub fn insert(&mut self, index: usize) -> bool {
        assert!(
            index < self.capacity,
            "index {index} is past the capacity {}",
            self.capacity
        );
        let word = &mut self.words[index / WORD_BITS];
        let added = *word & bit(index) == 0;
        *word |= bit(index);
        added
    }

    /// The number of indices in the set.
    pub fn count(&self) -> usize {
        self.words
            .iter()
            .map(|word| word.count_ones() as usize)
            .sum()
    }
}

/// The bit of `index` within its word.
fn bit(index: usize) -> u64 {
    1 << (index % WORD_BITS)
}
