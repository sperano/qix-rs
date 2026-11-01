//! The game state machine: the title screen, play, the pause, the end of
//! a level, the end of the game and the entry of a high score, around
//! one [`Game`] and the [`HighScores`] table.

use crate::event::Event;
use crate::game::Game;
use crate::high_score::{HighScore, HighScores};
use crate::initials::InitialsEntry;
use crate::input::InputState;
use crate::rules::Rules;

/// Where the session stands, which decides what a tick does.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GameState {
    /// The title screen, before a game starts: confirm starts it.
    Title,
    /// The game ticks: pause pauses it, a completed level or the loss of
    /// the last life ends it.
    Playing,
    /// The game is paused and does not tick: pause resumes it.
    Paused,
    /// The level is complete, as its [`Event::LevelComplete`] reported:
    /// confirm starts the next one.
    LevelComplete {
        /// The board's claimed fraction at the end, from 0 to 1.
        fraction: f32,
        /// The level bonus scored, zero after a split.
        bonus: u32,
        /// The level ended by splitting the Qix apart.
        split: bool,
    },
    /// The last life is lost: confirm enters the initials of a score that
    /// makes the high score table, or goes back to the title screen with
    /// a new game.
    GameOver,
    /// The initials of the game's score are being entered: once they are,
    /// the score goes into the table and the title screen shows a new
    /// game.
    HighScoreEntry(InitialsEntry),
}

/// A [`Game`], the [`GameState`] it is in, the high score table and the
/// best score so far.
#[derive(Debug)]
pub struct Session {
    state: GameState,
    game: Game,
    high_scores: HighScores,
    high_score: u32,
    /// The last tick of the session ticked the game.
    played: bool,
}

impl Session {
    /// A session on the title screen, which starts `game` on confirm,
    /// with an empty high score table.
    pub fn new(game: Game) -> Self {
        Self::with_high_scores(game, HighScores::default())
    }

    /// A session on the title screen, which starts `game` on confirm,
    /// with the table `high_scores` of earlier sessions.
    pub fn with_high_scores(game: Game, high_scores: HighScores) -> Self {
        Self {
            state: GameState::Title,
            game,
            high_score: high_scores.best(),
            high_scores,
            played: false,
        }
    }

    /// The state the session is in.
    pub fn state(&self) -> GameState {
        self.state
    }

    /// The game being played, or about to be on the title screen.
    pub fn game(&self) -> &Game {
        &self.game
    }

    /// The best score of the high score table and of every game of the
    /// session, the one being played included: it rises with the score as
    /// soon as the score passes it.
    pub fn high_score(&self) -> u32 {
        self.high_score
    }

    /// The high score table, with the scores entered this session.
    pub fn high_scores(&self) -> &HighScores {
        &self.high_scores
    }

    /// The events the game raised during the last tick of the session,
    /// in the order they happened; none when that tick did not tick the
    /// game, even though [`Game::events`] still holds those of its last
    /// tick.
    pub fn events(&self) -> &[Event] {
        if self.played { self.game.events() } else { &[] }
    }

    /// Sets the rules of the games to come (see [`Game::set_rules`]): on
    /// the title screen, the game it shows restarts with them; any other
    /// game keeps playing by its own.
    pub fn set_rules(&mut self, rules: Rules) {
        self.game.set_rules(rules);
        if self.state == GameState::Title {
            self.game.restart();
        }
    }

    /// Advances the session by one step: [`Game::tick`] with `input`
    /// while [`GameState::Playing`], or the transition `input` asks for.
    ///
    /// A tick that changes the state with a press does not tick the
    /// game: the pause stops it before it moves, and the game a confirm
    /// starts (from the title, or the next level) first moves on the
    /// tick after it. A tick of the game raising [`Event::GameOver`] or
    /// [`Event::LevelComplete`] ends play. Out of play, only those presses
    /// do anything, but for [`GameState::HighScoreEntry`], which hands
    /// every tick's input to its [`InitialsEntry`]: nothing ticks in
    /// [`GameState::Paused`]. Every tick raises the high score to the
    /// game's score if that is higher.
    pub fn tick(&mut self, input: &InputState) {
        self.played = false;
        self.state = match self.state {
            GameState::Title if input.confirm => GameState::Playing,
            GameState::Playing if input.pause => GameState::Paused,
            GameState::Playing => self.play(input),
            GameState::Paused if input.pause => GameState::Playing,
            GameState::LevelComplete { .. } if input.confirm => {
                self.game.start_level(self.game.level() + 1);
                GameState::Playing
            }
            GameState::GameOver if input.confirm => self.end_game(),
            GameState::HighScoreEntry(mut entry) => match entry.tick(input) {
                Some(name) => {
                    let score = self.game.score().points();
                    self.high_scores.insert(HighScore { name, score });
                    self.restart()
                }
                None => GameState::HighScoreEntry(entry),
            },
            state => state,
        };
        self.high_score = self.high_score.max(self.game.score().points());
    }

    /// The state after the game over screen: the initials entry when the
    /// score makes the table, the title with a new game otherwise.
    fn end_game(&mut self) -> GameState {
        if self.high_scores.qualifies(self.game.score().points()) {
            GameState::HighScoreEntry(InitialsEntry::new())
        } else {
            self.restart()
        }
    }

    /// Starts a new game and returns the title screen that shows it.
    fn restart(&mut self) -> GameState {
        self.game.restart();
        GameState::Title
    }

    /// Ticks the game with `input` and returns the state its events lead
    /// to.
    fn play(&mut self, input: &InputState) -> GameState {
        self.game.tick(input);
        self.played = true;
        self.game
            .events()
            .iter()
            .find_map(|event| match *event {
                Event::GameOver => Some(GameState::GameOver),
                Event::LevelComplete {
                    fraction,
                    bonus,
                    split,
                } => Some(GameState::LevelComplete {
                    fraction,
                    bonus,
                    split,
                }),
                Event::Draw(_) | Event::Claim { .. } | Event::Death(_) | Event::ExtraLife => None,
            })
            .unwrap_or(GameState::Playing)
    }
}
