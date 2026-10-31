//! The game state and its fixed-step update.

mod level_end;
mod level_start;
mod scoring;

use crate::board::{Board, DrawKind};
use crate::collision::qix_hits_player;
use crate::event::{DeathCause, Event};
use crate::fuse::{Fuse, FuseTimer};
use crate::geometry::{Dir, Point};
use crate::input::InputState;
use crate::level::ClaimThreshold;
use crate::marker::{Marker, MarkerState};
use crate::qix::Qix;
use crate::rng::Rng;
use crate::score::{Score, score_for_claim};
use crate::sparx::{Sparx, SparxKind};
use crate::sparx_meter::{MeterExpiry, SparxMeter};
use crate::stix::{Stix, legal_moves, legal_step};
use crate::tuning::{
    EXTRA_SPARX, FIRST_LEVEL, MARKER_STEP_TICKS, RESPAWN_GRACE_TICKS, SLOW_STEP_TICKS,
    SPARX_METER_TICKS, SPARX_STEP_TICKS, STARTING_LIVES, STARTING_MULTIPLIER,
    SUPER_SPARX_STEP_TICKS,
};

/// The whole game, advanced one fixed step at a time.
#[derive(Debug)]
pub struct Game {
    ticks: u64,
    /// The level being played, from [`FIRST_LEVEL`] on.
    level: u32,
    board: Board,
    marker: Marker,
    /// The stix being drawn: `Some` exactly while the marker is
    /// [`MarkerState::Drawing`].
    stix: Option<Stix>,
    /// Ticks a direction has been held since the marker's last step.
    marker_held_ticks: u32,
    /// The fuse of the stix, out while the marker steps.
    fuse: FuseTimer,
    qix: Vec<Qix>,
    sparx: Vec<Sparx>,
    /// Ticks since the Sparx on the border last stepped.
    sparx_ticks: u32,
    /// Ticks since the Sparx chasing along the stix last stepped.
    stix_sparx_ticks: u32,
    /// The kinds of the Sparx the level started with, the first of
    /// `sparx`: what a reset of the meter takes them back to.
    level_sparx: Vec<SparxKind>,
    /// Counts down to more Sparx, then to Super Sparx.
    sparx_meter: SparxMeter,
    /// Every random choice of the game comes from here.
    rng: Rng,
    /// Lives left, the one being played included; the game is over at
    /// zero.
    lives: u32,
    /// Ticks of the grace period after a death still to run.
    grace_ticks: u32,
    /// The points scored so far.
    score: Score,
    /// The multiplier of the points every claim scores.
    multiplier: u32,
    /// The claimed fraction that completes the level.
    threshold: ClaimThreshold,
    /// A claim reached the threshold or split the Qix apart: the game no
    /// longer ticks.
    level_complete: bool,
    /// What the last tick raised.
    events: Vec<Event>,
}

impl Game {
    /// A game at [`FIRST_LEVEL`] on `board`, with the marker at the
    /// centre of its bottom edge, `qix`, which must lie on free cells, and
    /// `sparx`, which must lie on the border, drawing from `rng`; its
    /// Sparx meter counts down the first level's [`SPARX_METER_TICKS`].
    pub fn with_board(board: Board, qix: Vec<Qix>, sparx: Vec<Sparx>, rng: Rng) -> Self {
        let marker = Marker::start(&board);
        Self {
            ticks: 0,
            level: FIRST_LEVEL,
            board,
            marker,
            stix: None,
            marker_held_ticks: 0,
            fuse: FuseTimer::default(),
            qix,
            level_sparx: sparx.iter().map(|sparx| sparx.kind).collect(),
            sparx,
            sparx_ticks: 0,
            stix_sparx_ticks: 0,
            sparx_meter: SparxMeter::new(SPARX_METER_TICKS),
            rng,
            lives: STARTING_LIVES,
            grace_ticks: 0,
            score: Score::default(),
            multiplier: STARTING_MULTIPLIER,
            threshold: ClaimThreshold::default(),
            level_complete: false,
            events: Vec::new(),
        }
    }

    /// Advances the game by one step of `1 / TICK_HZ` seconds: the marker
    /// moves, then the fuse burns, then each Qix moves, then the Sparx
    /// meter counts down (see [`Game::sparx_meter`]), then the Sparx; a
    /// Qix touching the stix or the drawing marker kills the player, else
    /// a Sparx touching the marker does, else the fuse at the marker; one
    /// death at most per tick. Once the game is over or the level
    /// complete, a tick does nothing.
    ///
    /// A death raises [`Event::Death`] and takes a life, and the last one
    /// [`Event::GameOver`] too: the stix and its fuse are cleared, the
    /// marker is back on the border where the stix began and a Sparx left
    /// off the border snaps back to it. For
    /// [`RESPAWN_GRACE_TICKS`] ticks after it, the Sparx cannot kill and
    /// the marker cannot draw. A death also resets the Sparx meter.
    ///
    /// A stix that completes the level (a claim reaching the threshold,
    /// or a split; see [`Game::is_level_complete`]) ends the tick: nothing
    /// moves after it.
    pub fn tick(&mut self, input: &InputState) {
        self.events.clear();
        if self.is_over() || self.level_complete {
            return;
        }
        self.ticks += 1;
        let grace = self.grace_ticks > 0;
        self.grace_ticks = self.grace_ticks.saturating_sub(1);
        let marker_from = self.marker.pos;
        self.move_marker(input, grace);
        if self.level_complete {
            return;
        }
        let fuse_hit = self.burn_fuse(marker_from);
        for qix in &mut self.qix {
            qix.tick(&self.board, &mut self.rng);
        }
        self.run_sparx_meter();
        let sparx_hit = self.move_sparx(marker_from) && !grace;
        let cause = if self.qix_hits_player() {
            Some(DeathCause::Qix)
        } else if sparx_hit {
            Some(DeathCause::Sparx)
        } else {
            fuse_hit.then_some(DeathCause::Fuse)
        };
        if let Some(cause) = cause {
            self.die(cause);
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

    /// The level being played, from [`FIRST_LEVEL`] on.
    pub fn level(&self) -> u32 {
        self.level
    }

    /// The Qix, one or two depending on the level.
    pub fn qix(&self) -> &[Qix] {
        &self.qix
    }

    /// The Sparx.
    pub fn sparx(&self) -> &[Sparx] {
        &self.sparx
    }

    /// The Sparx meter. Its first expiry releases [`EXTRA_SPARX`] more
    /// Sparx at the top centre, its second turns every Sparx
    /// [`SparxKind::Super`]. A death resets it and undoes both: the
    /// released Sparx are gone and the others are back to the kinds the
    /// level started them with.
    pub fn sparx_meter(&self) -> SparxMeter {
        self.sparx_meter
    }

    /// Replaces the Sparx meter, from the next tick on.
    pub fn set_sparx_meter(&mut self, meter: SparxMeter) {
        self.sparx_meter = meter;
    }

    /// The lives left, the one being played included: see
    /// [`Game::score`] for the extra ones.
    pub fn lives(&self) -> u32 {
        self.lives
    }

    /// Ticks of the grace period after a death still to run, the next
    /// one included; zero outside it.
    pub fn grace_ticks(&self) -> u32 {
        self.grace_ticks
    }

    /// The points scored so far. Each time they cross a multiple of
    /// [`EXTRA_LIFE_EVERY`](crate::tuning::EXTRA_LIFE_EVERY) the player
    /// earns a life, up to [`MAX_LIVES`](crate::tuning::MAX_LIVES), and
    /// the tick raises [`Event::ExtraLife`].
    pub fn score(&self) -> Score {
        self.score
    }

    /// The claimed fraction of the board that completes the level.
    pub fn claim_threshold(&self) -> ClaimThreshold {
        self.threshold
    }

    /// Sets the claimed fraction of the board that completes the level,
    /// checked from the next claim on.
    pub fn set_claim_threshold(&mut self, threshold: ClaimThreshold) {
        self.threshold = threshold;
    }

    /// A claim reached the claim threshold, or a stix split the Qix
    /// apart (see [`Game::multiplier`]): the game no longer ticks.
    pub fn is_level_complete(&self) -> bool {
        self.level_complete
    }

    /// The last life is lost: the game no longer ticks.
    pub fn is_over(&self) -> bool {
        self.lives == 0
    }

    /// The events the last tick raised, in the order they happened.
    pub fn events(&self) -> &[Event] {
        &self.events
    }

    /// Raises [`Event::Death`] for `cause` and takes a life: the stix and
    /// its fuse are cleared, the marker is back on the border where the
    /// stix began (or stays where it is on the border), a Sparx that was on
    /// the stix snaps to the border and the grace period starts. Taking the last life raises [`Event::GameOver`]
    /// instead of starting a grace period.
    fn die(&mut self, cause: DeathCause) {
        self.events.push(Event::Death(cause));
        self.lives -= 1;
        if let Some(stix) = self.stix.take() {
            self.marker.pos = stix.start();
        }
        self.marker.state = MarkerState::OnBorder;
        self.marker_held_ticks = 0;
        self.fuse.put_out();
        self.reset_sparx_meter();
        self.snap_sparx_to_border();
        if self.is_over() {
            self.events.push(Event::GameOver);
        } else {
            self.grace_ticks = RESPAWN_GRACE_TICKS;
        }
    }

    /// Counts the Sparx meter down one tick and applies its expiry, if
    /// this tick brings one.
    fn run_sparx_meter(&mut self) {
        match self.sparx_meter.tick() {
            Some(MeterExpiry::Release) => self
                .sparx
                .extend(Sparx::spawn_at_top(&self.board, EXTRA_SPARX)),
            Some(MeterExpiry::Super) => {
                for sparx in &mut self.sparx {
                    sparx.kind = SparxKind::Super;
                }
            }
            None => {}
        }
    }

    /// Fills the Sparx meter again and undoes its expiries: the Sparx it
    /// released are removed and the level's own are back to their kinds.
    fn reset_sparx_meter(&mut self) {
        self.sparx_meter.reset();
        self.sparx.truncate(self.level_sparx.len());
        for (sparx, &kind) in self.sparx.iter_mut().zip(&self.level_sparx) {
            sparx.kind = kind;
        }
    }

    /// A Qix touches the stix or the marker drawing it; a marker on the
    /// border has no stix and is never hit.
    fn qix_hits_player(&self) -> bool {
        self.stix
            .as_ref()
            .is_some_and(|stix| qix_hits_player(&self.qix, stix, self.marker.pos))
    }

    /// Burns the fuse one tick of a marker standing still on the stix, or
    /// puts it out when the marker stepped from `marker_from` this tick or
    /// has no stix; a marker caught in a dead end lights it at once.
    /// Returns whether the fuse is at the marker, which kills it.
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

    /// Steps every Sparx on the border once every [`SPARX_STEP_TICKS`]
    /// ticks and every one chasing along the stix (see
    /// [`Sparx::chases_along`]) once every [`SUPER_SPARX_STEP_TICKS`].
    /// Returns whether one of them touched the marker, which moved from
    /// `marker_from` this tick: see [`Sparx::hits_marker`].
    fn move_sparx(&mut self, marker_from: Point) -> bool {
        let border_due = count_to(&mut self.sparx_ticks, SPARX_STEP_TICKS);
        let stix_due = count_to(&mut self.stix_sparx_ticks, SUPER_SPARX_STEP_TICKS);
        let mut hit = false;
        for sparx in &mut self.sparx {
            let from = sparx.pos;
            match self.stix.as_ref().filter(|stix| sparx.chases_along(stix)) {
                Some(stix) if stix_due => sparx.step_along_stix(stix),
                None if border_due => sparx.step(&self.board, &mut self.rng),
                _ => {}
            }
            hit |= sparx.hits_marker(from, marker_from, self.marker.pos);
        }
        hit
    }

    /// Moves every Sparx off the border to the border point nearest it.
    fn snap_sparx_to_border(&mut self) {
        for sparx in &mut self.sparx {
            sparx.snap_to_border(&self.board);
        }
    }

    /// Steps the marker once every [`Game::step_ticks`] ticks the
    /// direction is held.
    ///
    /// While drawing, the marker stands still on the stix as long as no
    /// draw button is held; pressing one again restarts the step count.
    /// During the `grace` period the draw buttons are ignored.
    fn move_marker(&mut self, input: &InputState, grace: bool) {
        let draw = input.draw_kind().filter(|_| !grace);
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
    /// from every Qix is claimed and scored, the stix is dropped and its fuse
    /// put out, the marker is back on the border, at the closing point, and
    /// a Sparx the claim left off the border (one that was on the stix, or
    /// one the claim cut off) snaps back to it. A stix leaving the Qix in
    /// different areas, or a claim taking the board to the threshold,
    /// completes the level.
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
            let close = self.board.close_stix(&stix, &self.qix);
            if let Some(claim) = &close.claim {
                self.add_score(score_for_claim(claim, self.multiplier));
            }
            self.end_level_after(&close);
            self.marker.state = MarkerState::OnBorder;
            self.fuse.put_out();
            self.snap_sparx_to_border();
        } else {
            self.marker.state = MarkerState::Drawing;
            self.stix = Some(stix);
        }
    }
}

/// Counts one more tick on `ticks` and returns whether it reached
/// `period`, which starts the count again.
fn count_to(ticks: &mut u32, period: u32) -> bool {
    *ticks += 1;
    let due = *ticks >= period;
    if due {
        *ticks = 0;
    }
    due
}
