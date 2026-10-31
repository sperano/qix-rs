//! Adding points to the score, and the extra lives they earn.

use super::Game;
use crate::event::Event;
use crate::score::extra_lives;

impl Game {
    /// Adds `points` to the score and grants the extra lives it earns
    /// (see [`extra_lives`]), raising [`Event::ExtraLife`] for each.
    pub(super) fn add_score(&mut self, points: u32) {
        let before = self.score;
        self.score.add(points);
        let earned = extra_lives(before, self.score, self.lives);
        self.lives += earned;
        for _ in 0..earned {
            self.events.push(Event::ExtraLife);
        }
    }
}
