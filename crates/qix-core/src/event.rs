//! What a tick of the game reports to the application.

/// Something that happened during a tick.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Event {
    /// The player lost a life.
    Death(DeathCause),
    /// The player lost the last life, right after its
    /// [`Event::Death`]: the game no longer ticks.
    GameOver,
    /// A claim took the board to the claim threshold or above, and the
    /// level bonus was scored, or a stix split the Qix apart, which
    /// scores no bonus but raises the multiplier: the game no longer
    /// ticks.
    LevelComplete {
        /// The board's claimed fraction after the claim, from 0 to 1.
        fraction: f32,
        /// The level bonus scored, zero after a split.
        bonus: u32,
        /// The stix left the Qix in different free areas.
        split: bool,
    },
}

/// What killed the player.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DeathCause {
    /// The Qix touched the stix or the drawing marker.
    Qix,
    /// A Sparx reached the marker.
    Sparx,
    /// The fuse burnt along the stix to the marker.
    Fuse,
}
