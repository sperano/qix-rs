//! Starting a game, and each of its levels, from the level table.

use super::Game;
use crate::board::Board;
use crate::level::LevelConfig;
use crate::qix::Qix;
use crate::rng::Rng;
use crate::sparx::Sparx;
use crate::sparx_meter::SparxMeter;
use crate::tuning::{BOARD_H, BOARD_W, FIRST_LEVEL};
use crate::vec2::Vec2;

impl Game {
    /// A game that has not ticked yet, at the start of [`FIRST_LEVEL`]
    /// on a free `BOARD_W x BOARD_H` board: see [`Game::start_level`].
    /// Every random choice is drawn from `rng`.
    pub fn new(rng: Rng) -> Self {
        let board = Board::new(BOARD_W, BOARD_H);
        let mut game = Self::with_board(board, Vec::new(), Vec::new(), rng);
        game.start_level(FIRST_LEVEL);
        game
    }

    /// Starts a new game on a free board of the same size, as
    /// [`Game::new`] does: [`FIRST_LEVEL`], every life, no score, the
    /// starting multiplier, the default claim threshold and no tick run.
    /// Only the random sequence carries on.
    pub fn restart(&mut self) {
        let board = Board::new(self.board.width(), self.board.height());
        *self = Self::with_board(board, Vec::new(), Vec::new(), self.rng.clone());
        self.start_level(FIRST_LEVEL);
    }

    /// Starts level `level` (a level below [`FIRST_LEVEL`] starts the
    /// first) as [`LevelConfig::for_level`] sets it up: a free board of
    /// the same size, the marker at the centre of its bottom edge, the
    /// Qix spread evenly along its middle row, the
    /// [`Sparx::spawn_level`] on its top edge and a full Sparx meter.
    ///
    /// The score, the lives, the multiplier, the claim threshold, the
    /// tick count and the random sequence carry on; the stix, the fuse,
    /// a grace period, the completed level and the last tick's events are
    /// gone.
    pub fn start_level(&mut self, level: u32) {
        let level = level.max(FIRST_LEVEL);
        let config = LevelConfig::for_level(level);
        let board = Board::new(self.board.width(), self.board.height());
        let qix = spawn_qix(&board, &config, &mut self.rng);
        let sparx = Sparx::spawn_level(&board, config.sparx_count);
        let rng = self.rng.clone();
        *self = Self {
            ticks: self.ticks,
            level,
            sparx_meter: SparxMeter::new(config.meter_ticks),
            lives: self.lives,
            score: self.score,
            multiplier: self.multiplier,
            threshold: self.threshold,
            ..Self::with_board(board, qix, sparx, rng)
        };
    }
}

/// The `config.qix_count` Qix of a level on `board`, drawn from `rng`:
/// spread evenly along the board's middle row, the ends excluded, so a
/// single one is at its centre, each kept within `config.qix_speed`.
fn spawn_qix(board: &Board, config: &LevelConfig, rng: &mut Rng) -> Vec<Qix> {
    let (width, height) = (f32::from(board.width()), f32::from(board.height()));
    let slots = (config.qix_count + 1) as f32;
    (1..=config.qix_count)
        .map(|slot| {
            let centre = Vec2::new(width * slot as f32 / slots, height * 0.5);
            Qix::spawn_at(centre, config.qix_speed.clone(), rng)
        })
        .collect()
}
