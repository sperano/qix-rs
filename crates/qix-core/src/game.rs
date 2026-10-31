//! The game state and its fixed-step update.

use crate::board::{Board, DrawKind};
use crate::collision::{qix_hits_marker, qix_hits_stix};
use crate::event::{DeathCause, Event};
use crate::fuse::{Fuse, FuseTimer};
use crate::geometry::{Dir, Point};
use crate::input::InputState;
use crate::marker::{Marker, MarkerState};
use crate::qix::Qix;
use crate::rng::Rng;
use crate::sparx::Sparx;
use crate::stix::{Stix, legal_moves, legal_step};
use crate::tuning::{BOARD_H, BOARD_W, MARKER_STEP_TICKS, SLOW_STEP_TICKS, SPARX_STEP_TICKS};

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
    /// The fuse of the stix, out while the marker steps.
    fuse: FuseTimer,
    qix: Qix,
    sparx: Vec<Sparx>,
    /// Ticks since the Sparx last stepped.
    sparx_ticks: u32,
    /// Every random choice of the game comes from here.
    rng: Rng,
    /// What the last tick raised.
    events: Vec<Event>,
}

impl Game {
    /// A game that has not ticked yet: a free `BOARD_W x BOARD_H` board
    /// with the marker at the centre of its bottom edge, a Qix spawned at
    /// its centre and the [`Sparx::spawn_pair`] at the centre of its top
    /// edge, every random choice drawn from `rng`.
    pub fn new(mut rng: Rng) -> Self {
        let board = Board::new(BOARD_W, BOARD_H);
        let qix = Qix::spawn(&board, &mut rng);
        let sparx = Sparx::spawn_pair(&board).to_vec();
        Self::with_board(board, qix, sparx, rng)
    }

    /// A game on `board`, with the marker at the centre of its bottom
    /// edge, `qix`, which must lie on free cells, and `sparx`, which must
    /// lie on the border, drawing from `rng`.
    pub fn with_board(board: Board, qix: Qix, sparx: Vec<Sparx>, rng: Rng) -> Self {
        let marker = Marker::start(&board);
        Self {
            ticks: 0,
            board,
            marker,
            stix: None,
            marker_held_ticks: 0,
            fuse: FuseTimer::default(),
            qix,
            sparx,
            sparx_ticks: 0,
            rng,
            events: Vec::new(),
        }
    }

    /// Advances the game by one step of `1 / TICK_HZ` seconds: the marker
    /// moves, then the fuse burns, then the Qix moves, then the Sparx; a
    /// Qix touching the stix or the drawing marker raises
    /// [`Event::Death`], then so does a Sparx touching the marker, then
    /// the fuse at the marker.
    pub fn tick(&mut self, input: &InputState) {
        self.ticks += 1;
        self.events.clear();
        let marker_from = self.marker.pos;
        self.move_marker(input);
        let fuse_hit = self.burn_fuse(marker_from);
        self.qix.tick(&self.board, &mut self.rng);
        let sparx_hit = self.move_sparx(marker_from);
        if self.qix_hits_player() {
            self.events.push(Event::Death(DeathCause::Qix));
        }
        if sparx_hit {
            self.events.push(Event::Death(DeathCause::Sparx));
        }
        if fuse_hit {
            self.events.push(Event::Death(DeathCause::Fuse));
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

    /// The fuse burning along the stix, if it is lit.
    pub fn fuse(&self) -> Option<Fuse> {
        self.fuse.fuse()
    }

    /// The Qix.
    pub fn qix(&self) -> &Qix {
        &self.qix
    }

    /// The Sparx.
    pub fn sparx(&self) -> &[Sparx] {
        &self.sparx
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

    /// Burns the fuse one tick of a marker standing still on the stix, or
    /// puts it out when the marker stepped from `marker_from` this tick or
    /// has no stix; a marker caught in a dead end lights it at once.
    /// Returns whether the fuse is at the marker: it stays there, raising
    /// a death every tick, until the marker steps or the stix closes.
    fn burn_fuse(&mut self, marker_from: Point) -> bool {
        let Some(stix) = &self.stix else {
            self.fuse.put_out();
            return false;
        };
        let head = stix.points().len() - 1;
        if self.marker.pos == marker_from {
            self.fuse.stand(head);
        } else {
            self.fuse.put_out();
        }
        if self.fuse.fuse().is_none() && legal_moves(&self.marker, stix, &self.board).is_empty() {
            self.fuse.light();
        }
        self.fuse.reached(head)
    }

    /// Steps every Sparx once every [`SPARX_STEP_TICKS`] ticks. Returns
    /// whether one of them touched the marker, which moved from
    /// `marker_from` this tick: see [`Sparx::hits_marker`].
    fn move_sparx(&mut self, marker_from: Point) -> bool {
        self.sparx_ticks += 1;
        let due = self.sparx_ticks >= SPARX_STEP_TICKS;
        if due {
            self.sparx_ticks = 0;
        }
        let mut hit = false;
        for sparx in &mut self.sparx {
            let from = sparx.pos;
            if due {
                sparx.step(&self.board, &mut self.rng);
            }
            hit |= sparx.hits_marker(from, marker_from, self.marker.pos);
        }
        hit
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
    /// from [`Game::claim_seed`] is claimed, the stix is dropped, the
    /// marker is back on the border, at the closing point, and a Sparx
    /// the claim took off the border snaps back to it.
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
            for sparx in &mut self.sparx {
                sparx.snap_to_border(&self.board);
            }
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
