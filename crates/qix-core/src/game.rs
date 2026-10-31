//! The game state and its fixed-step update.

use crate::board::{Board, DrawKind};
use crate::collision::{qix_hits_marker, qix_hits_stix};
use crate::event::{DeathCause, Event};
use crate::geometry::Dir;
use crate::input::InputState;
use crate::marker::{Marker, MarkerState};
use crate::qix::Qix;
use crate::rng::Rng;
use crate::stix::{Stix, legal_step};
use crate::tuning::{BOARD_H, BOARD_W, MARKER_STEP_TICKS, SLOW_STEP_TICKS};

/// The whole game, advanced one fixed step at a time.
#[derive(Debug)]
pub struct Game {
    ticks: u64,
    board: Board,
    marker: Marker,
    /// The stix being drawn: `Some` exactly while the marker is
    /// [`MarkerState::Drawing`].
    stix: Option<Stix>,
    /// Ticks a direction has been held since the marker's last step.
    marker_held_ticks: u32,
    qix: Qix,
    /// Every random choice of the game comes from here.
    rng: Rng,
    /// What the last tick raised.
    events: Vec<Event>,
}

impl Game {
    /// A game that has not ticked yet: a free `BOARD_W x BOARD_H` board
    /// with the marker at the centre of its bottom edge and a Qix spawned
    /// at its centre, every random choice drawn from `rng`.
    pub fn new(mut rng: Rng) -> Self {
        let board = Board::new(BOARD_W, BOARD_H);
        let qix = Qix::spawn(&board, &mut rng);
        Self::with_board(board, qix, rng)
    }

    /// A game on `board`, with the marker at the centre of its bottom edge
    /// and `qix`, which must lie on free cells, drawing from `rng`.
    pub fn with_board(board: Board, qix: Qix, rng: Rng) -> Self {
        let marker = Marker::start(&board);
        Self {
            ticks: 0,
            board,
            marker,
            stix: None,
            marker_held_ticks: 0,
            qix,
            rng,
            events: Vec::new(),
        }
    }

    /// Advances the game by one step of `1 / TICK_HZ` seconds: the marker
    /// moves, then the Qix, then a Qix touching the stix or the drawing
    /// marker raises [`Event::Death`].
    pub fn tick(&mut self, input: &InputState) {
        self.ticks += 1;
        self.events.clear();
        self.move_marker(input);
        self.qix.tick(&self.board, &mut self.rng);
        if self.qix_hits_player() {
            self.events.push(Event::Death(DeathCause::Qix));
        }
    }

    /// The number of steps run so far.
    pub fn ticks(&self) -> u64 {
        self.ticks
    }

    /// The board.
    pub fn board(&self) -> &Board {
        &self.board
    }

    /// The player's marker.
    pub fn marker(&self) -> Marker {
        self.marker
    }

    /// The stix being drawn, if the marker is drawing one.
    pub fn stix(&self) -> Option<&Stix> {
        self.stix.as_ref()
    }

    /// The Qix.
    pub fn qix(&self) -> &Qix {
        &self.qix
    }

    /// The events the last tick raised, in the order they happened.
    pub fn events(&self) -> &[Event] {
        &self.events
    }

    /// The Qix touches the stix or the marker drawing it; a marker on the
    /// border has no stix and is never hit.
    fn qix_hits_player(&self) -> bool {
        self.stix.as_ref().is_some_and(|stix| {
            qix_hits_stix(&self.qix, stix) || qix_hits_marker(&self.qix, self.marker.pos)
        })
    }

    /// Steps the marker once every [`Game::step_ticks`] ticks the
    /// direction is held.
    ///
    /// While drawing, the marker stands still on the stix as long as no
    /// draw button is held; pressing one again restarts the step count.
    fn move_marker(&mut self, input: &InputState) {
        let draw = input.draw_kind();
        let stopped = self.marker.state == MarkerState::Drawing && draw.is_none();
        let Some(dir) = input.dir.filter(|_| !stopped) else {
            self.marker_held_ticks = 0;
            return;
        };
        self.marker_held_ticks += 1;
        if self.marker_held_ticks < self.step_ticks(dir, draw) {
            return;
        }
        self.marker_held_ticks = 0;
        let along_border = self.marker.state == MarkerState::OnBorder
            && self.marker.step_along_border(&self.board, dir);
        if let Some(kind) = draw.filter(|_| !along_border) {
            self.draw(dir, kind);
        }
    }

    /// The ticks a step towards `dir` takes with the draw button `draw`
    /// held: [`MARKER_STEP_TICKS`] along the border, whatever the button,
    /// and the speed of the button for a step of the stix.
    fn step_ticks(&self, dir: Dir, draw: Option<DrawKind>) -> u32 {
        let along_border = self.marker.state == MarkerState::OnBorder
            && self.marker.border_step(&self.board, dir).is_some();
        match draw {
            Some(DrawKind::Slow) if !along_border => SLOW_STEP_TICKS,
            _ => MARKER_STEP_TICKS,
        }
    }

    /// Draws one step of the stix towards `dir` with `kind`, starting a
    /// stix when the marker is on the border; an illegal step leaves the
    /// marker where it is.
    ///
    /// A step onto a border point closes the stix: the area it cuts off
    /// from [`Game::claim_seed`] is claimed, the stix is dropped and the
    /// marker is back on the border, at the closing point.
    fn draw(&mut self, dir: Dir, kind: DrawKind) {
        let mut stix = self
            .stix
            .take()
            .unwrap_or_else(|| Stix::new(self.marker.pos));
        let Some(next) = legal_step(&self.marker, &stix, &self.board, dir) else {
            self.stix = (self.marker.state == MarkerState::Drawing).then_some(stix);
            return;
        };
        stix.push(next, kind);
        self.marker.pos = next;
        if self.board.is_border(next) {
            self.board.close_stix(&stix, self.claim_seed());
            self.marker.state = MarkerState::OnBorder;
        } else {
            self.marker.state = MarkerState::Drawing;
            self.stix = Some(stix);
        }
    }

    /// The cell whose area stays free when a stix closes: the centre of
    /// the board, until the Qix takes its place.
    fn claim_seed(&self) -> (u16, u16) {
        (self.board.width() / 2, self.board.height() / 2)
    }
}
