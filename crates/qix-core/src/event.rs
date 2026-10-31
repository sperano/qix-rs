//! What a tick of the game reports to the application.

/// Something that happened during a tick.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Event {
    /// The player lost a life.
    Death(DeathCause),
    /// The player lost the last life, right after its
    /// [`Event::Death`]: the game no longer ticks.
    GameOver,
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
