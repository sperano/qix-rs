//! The Sparx meter: the countdown that releases more Sparx, then turns
//! every Sparx Super.

/// What the Sparx meter does when its countdown runs out.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MeterExpiry {
    /// The first expiry: more Sparx are released at the top centre.
    Release,
    /// The second expiry: every Sparx turns Super.
    Super,
}

impl MeterExpiry {
    /// The expiry after this one, `None` after the last.
    const fn next(self) -> Option<Self> {
        match self {
            Self::Release => Some(Self::Super),
            Self::Super => None,
        }
    }
}

/// Counts down twice per level, the same length each time: first to a
/// [`MeterExpiry::Release`], then to a [`MeterExpiry::Super`]; after
/// that it is spent and stays at zero until reset.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SparxMeter {
    /// Ticks of one countdown, at least one.
    length: u32,
    /// Ticks left of the current countdown.
    remaining: u32,
    /// What the current countdown leads to, `None` once spent.
    next: Option<MeterExpiry>,
}

impl SparxMeter {
    /// A full meter counting down `length` ticks (at least one) to its
    /// [`MeterExpiry::Release`].
    pub fn new(length: u32) -> Self {
        let length = length.max(1);
        Self {
            length,
            remaining: length,
            next: Some(MeterExpiry::Release),
        }
    }

    /// Counts one tick down; returns the expiry when this tick ends the
    /// countdown, which then starts over towards the next expiry. A spent
    /// meter does nothing.
    pub fn tick(&mut self) -> Option<MeterExpiry> {
        let expiry = self.next?;
        self.remaining -= 1;
        if self.remaining > 0 {
            return None;
        }
        self.next = expiry.next();
        if self.next.is_some() {
            self.remaining = self.length;
        }
        Some(expiry)
    }

    /// Fills the meter again, back to counting down to its
    /// [`MeterExpiry::Release`].
    pub fn reset(&mut self) {
        *self = Self::new(self.length);
    }

    /// Ticks of one countdown.
    pub const fn length(self) -> u32 {
        self.length
    }

    /// Ticks left of the current countdown; zero once spent.
    pub const fn remaining(self) -> u32 {
        self.remaining
    }

    /// What the current countdown leads to, `None` once both expiries
    /// have passed.
    pub const fn next_expiry(self) -> Option<MeterExpiry> {
        self.next
    }
}
