//! The seeded random number generator the game draws from.
//!
//! The application injects one into the core, so a seed fixes every
//! random choice and simulations and tests replay exactly.

/// SplitMix64 increment and output multipliers.
const SPLITMIX_GAMMA: u64 = 0x9E37_79B9_7F4A_7C15;
const SPLITMIX_MUL_1: u64 = 0xBF58_476D_1CE4_E5B9;
const SPLITMIX_MUL_2: u64 = 0x94D0_49BB_1331_11EB;

/// Bits of a random `u64` dropped to make an `f32` fraction: all but the
/// width of the `f32` significand, so every fraction is exact.
const F32_DROPPED_BITS: u32 = u64::BITS - f32::MANTISSA_DIGITS;

/// A SplitMix64 generator: tiny, fast, and the same sequence for a seed on
/// every platform.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rng {
    state: u64,
}

impl Rng {
    /// A generator whose sequence is fixed by `seed`.
    pub const fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    /// The next 64 random bits.
    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(SPLITMIX_GAMMA);
        let z = self.state;
        let z = (z ^ (z >> 30)).wrapping_mul(SPLITMIX_MUL_1);
        let z = (z ^ (z >> 27)).wrapping_mul(SPLITMIX_MUL_2);
        z ^ (z >> 31)
    }

    /// A uniform fraction in `[0, 1)`.
    pub fn next_f32(&mut self) -> f32 {
        let bits = self.next_u64() >> F32_DROPPED_BITS;
        bits as f32 / (1u64 << f32::MANTISSA_DIGITS) as f32
    }

    /// A uniform value in `[low, high)`; `low` itself when the two are
    /// equal.
    pub fn range_f32(&mut self, low: f32, high: f32) -> f32 {
        low + (high - low) * self.next_f32()
    }
}
