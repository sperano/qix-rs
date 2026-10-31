//! The game state machine: the title screen, play, the pause, the end of
//! a level and the end of the game, around one [`Game`].

use crate::event::Event;
use crate::game::Game;
use crate::input::InputState;

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
    /// The last life is lost: confirm goes back to the title screen with
    /// a new game.
    GameOver,
}

/// A [`Game`], the [`GameState`] it is in and the best score of the
/// session.
#[derive(Debug)]
pub struct Session {
    state: GameState,
    game: Game,
    high_score: u32,
}

impl Session {
    /// A session on the title screen, which starts `game` on confirm.
    pub fn new(game: Game) -> Self {
        Self {
            state: GameState::Title,
            game,
            high_score: 0,
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

    /// The best score of every game of the session, the one being played
    /// included: it rises with the score as soon as the score passes it,
    /// and starts at zero.
    pub fn high_score(&self) -> u32 {
        self.high_score
    }

    /// Advances the session by one step: [`Game::tick`] with `input`
    /// while [`GameState::Playing`], or the transition `input` asks for.
    ///
    /// A tick that changes the state with a press does not tick the
    /// game: the pause stops it before it moves, and the game a confirm
    /// starts (from the title, or the next level) first moves on the
    /// tick after it. A tick of the game raising [`Event::GameOver`] or
    /// [`Event::LevelComplete`] ends play. Out of play, only those presses
    /// do anything: nothing ticks in [`GameState::Paused`]. Every tick
    /// raises the high score to the game's score if that is higher.
    pub fn tick(&mut self, input: &InputState) {
        self.state = match self.state {
            GameState::Title if input.confirm => GameState::Playing,
            GameState::Playing if input.pause => GameState::Paused,
            GameState::Playing => self.play(input),
            GameState::Paused if input.pause => GameState::Playing,
            GameState::LevelComplete { .. } if input.confirm => {
                self.game.start_level(self.game.level() + 1);
                GameState::Playing
            }
            GameState::GameOver if input.confirm => {
                self.game.restart();
                GameState::Title
            }
            state => state,
        };
        self.high_score = self.high_score.max(self.game.score().points());
    }

    /// Ticks the game with `input` and returns the state its events lead
    /// to.
    fn play(&mut self, input: &InputState) -> GameState {
        self.game.tick(input);
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
                Event::Death(_) => None,
            })
            .unwrap_or(GameState::Playing)
    }
}
