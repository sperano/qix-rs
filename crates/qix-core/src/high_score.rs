//! The high score table: the best [`HIGH_SCORE_COUNT`] scores, each with
//! the initials of who made it.

use serde::{Deserialize, Serialize};

use crate::initials::Initials;
use crate::tuning::HIGH_SCORE_COUNT;

/// One line of the table: a score and who made it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HighScore {
    pub name: Initials,
    pub score: u32,
}

/// The best scores, best first, at most [`HIGH_SCORE_COUNT`] of them; a
/// score equal to one already in the table ranks below it.
///
/// It serialises as the list of its entries. Deserialising goes through
/// [`From<Vec<HighScore>>`], so a stored list edited by hand still makes
/// a valid table.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "Vec<HighScore>", into = "Vec<HighScore>")]
pub struct HighScores {
    entries: Vec<HighScore>,
}

impl HighScores {
    /// The entries, best first.
    pub fn entries(&self) -> &[HighScore] {
        &self.entries
    }

    /// The best score of the table, zero when it is empty.
    pub fn best(&self) -> u32 {
        self.entries.first().map_or(0, |entry| entry.score)
    }

    /// The index `score` would take in the table, or `None` when it does
    /// not make it: a score of zero never does, nor one no better than
    /// the last of a full table.
    pub fn rank(&self, score: u32) -> Option<usize> {
        let rank = self
            .entries
            .iter()
            .position(|entry| entry.score < score)
            .unwrap_or(self.entries.len());
        (score > 0 && rank < HIGH_SCORE_COUNT).then_some(rank)
    }

    /// Whether `score` makes it into the table.
    pub fn qualifies(&self, score: u32) -> bool {
        self.rank(score).is_some()
    }

    /// Puts `entry` at its rank, dropping the last entry of a full table,
    /// and returns the rank; leaves the table alone and returns `None`
    /// when the score does not qualify.
    pub fn insert(&mut self, entry: HighScore) -> Option<usize> {
        let rank = self.rank(entry.score)?;
        self.entries.insert(rank, entry);
        self.entries.truncate(HIGH_SCORE_COUNT);
        Some(rank)
    }
}

impl From<Vec<HighScore>> for HighScores {
    /// The table of `entries` in any order: sorted best first (equal
    /// scores keep their order), scores of zero dropped, and cut to
    /// [`HIGH_SCORE_COUNT`].
    fn from(mut entries: Vec<HighScore>) -> Self {
        entries.retain(|entry| entry.score > 0);
        entries.sort_by_key(|entry| std::cmp::Reverse(entry.score));
        entries.truncate(HIGH_SCORE_COUNT);
        Self { entries }
    }
}

impl From<HighScores> for Vec<HighScore> {
    fn from(table: HighScores) -> Self {
        table.entries
    }
}
