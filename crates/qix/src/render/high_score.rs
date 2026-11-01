//! The high score table as the title screen lists it, and the screen
//! that enters the initials of a new high score.

use qix_core::{Dir, Game, HighScores, InitialsEntry};

use crate::input::{CONFIRM_KEY, DIR_KEYS};
use crate::numbers::thousands;
use crate::render::screen::Screen;

/// The rows of `high_scores`, best first: rank, initials and score, with
/// the ranks and scores right-aligned in a monospace font.
pub fn table_rows(high_scores: &HighScores) -> Vec<String> {
    let entries = high_scores.entries();
    let scores: Vec<_> = entries.iter().map(|entry| thousands(entry.score)).collect();
    let score_width = scores.iter().map(String::len).max().unwrap_or(0);
    let rank_width = entries.len().to_string().len();
    entries
        .iter()
        .zip(&scores)
        .enumerate()
        .map(|(i, (entry, score))| {
            let rank = i + 1;
            format!("{rank:>rank_width$}. {}  {score:>score_width$}", entry.name)
        })
        .collect()
}

/// The initials of `entry` as one row, the letter under the cursor in
/// brackets: ` A [B] C `.
pub fn initials_row(entry: &InitialsEntry) -> String {
    entry
        .initials()
        .letters()
        .iter()
        .enumerate()
        .map(|(i, letter)| {
            if i == entry.cursor() {
                format!("[{letter}]")
            } else {
                format!(" {letter} ")
            }
        })
        .collect()
}

/// The screen entering the initials of `game`'s score, which makes
/// `high_scores`: the initials so far, the rank and the controls.
pub fn high_score_entry(game: &Game, high_scores: &HighScores, entry: &InitialsEntry) -> Screen {
    let points = game.score().points();
    let mut lines = vec![format!("Score {}", thousands(points))];
    if let Some(rank) = high_scores.rank(points) {
        lines[0].push_str(&format!(", rank {}", rank + 1));
    }
    lines.extend([
        String::new(),
        format!(
            "{}/{}: change the letter",
            key_name(Dir::Up),
            key_name(Dir::Down)
        ),
        format!("{}/{}: move", key_name(Dir::Left), key_name(Dir::Right)),
        format!("{}: next letter, then done", CONFIRM_KEY.name()),
    ]);
    Screen {
        heading: "NEW HIGH SCORE".to_owned(),
        table: vec![initials_row(entry)],
        lines,
    }
}

/// The name of the key that moves in `dir`.
fn key_name(dir: Dir) -> &'static str {
    DIR_KEYS
        .iter()
        .find(|&&(_, key_dir)| key_dir == dir)
        .map_or("?", |(key, _)| key.name())
}
