//! The Qix: a line segment drifting at random through the free area,
//! leaving a trail of its last positions.

use std::collections::VecDeque;
use std::f32::consts::TAU;
use std::ops::RangeInclusive;

use crate::board::Board;
use crate::rng::Rng;
use crate::segment;
use crate::tuning::{
    QIX_JITTER, QIX_LENGTH, QIX_MAX_SPIN, QIX_SPEED, QIX_SPIN_JITTER, QIX_TRAIL_LEN,
};
use crate::vec2::Vec2;

/// The Qix, in playfield units.
///
/// It keeps its length: each tick its midpoint moves by `vel` and the
/// segment turns about it by `spin`. As long as it starts on free cells,
/// every cell under its segment stays free.
#[derive(Clone, Debug, PartialEq)]
pub struct Qix {
    a: Vec2,
    b: Vec2,
    /// The distance from `a` to `b`, kept exact: turning the segment tick
    /// after tick would otherwise let rounding stretch or shrink it.
    length: f32,
    /// Displacement of the midpoint per tick.
    vel: Vec2,
    /// Turn of the segment about its midpoint per tick, in radians.
    spin: f32,
    /// The speeds the midpoint keeps within.
    speed: RangeInclusive<f32>,
    /// The last [`QIX_TRAIL_LEN`] segments, oldest first; the newest is
    /// the current one.
    trail: VecDeque<(Vec2, Vec2)>,
}

impl Qix {
    /// A Qix on the segment from `a` to `b`, with `vel` and `spin`, kept
    /// within the first level's [`QIX_SPEED`]; its trail holds that one
    /// segment.
    ///
    /// Nothing is checked: the caller puts it on free cells of the board
    /// it will tick on.
    pub fn new(a: Vec2, b: Vec2, vel: Vec2, spin: f32) -> Self {
        let mut trail = VecDeque::with_capacity(QIX_TRAIL_LEN);
        trail.push_back((a, b));
        Self {
            a,
            b,
            length: (b - a).length(),
            vel,
            spin,
            speed: QIX_SPEED,
            trail,
        }
    }

    /// A [`QIX_LENGTH`] Qix centred on `board`, at a random angle, heading
    /// a random way at a random speed within [`QIX_SPEED`], with a random
    /// spin.
    ///
    /// The cells it covers must be free.
    pub fn spawn(board: &Board, rng: &mut Rng) -> Self {
        let centre = Vec2::new(f32::from(board.width()), f32::from(board.height())) * 0.5;
        Self::spawn_at(centre, QIX_SPEED, rng)
    }

    /// A [`QIX_LENGTH`] Qix centred on `centre`, at a random angle,
    /// heading a random way at a random speed within `speed`, which it
    /// keeps within from then on, with a random spin.
    ///
    /// The cells it covers must be free.
    pub fn spawn_at(centre: Vec2, speed: RangeInclusive<f32>, rng: &mut Rng) -> Self {
        let half = Vec2::from_angle(rng.range_f32(0.0, TAU)) * (QIX_LENGTH * 0.5);
        let initial_speed = rng.range_f32(*speed.start(), *speed.end());
        let vel = Vec2::from_angle(rng.range_f32(0.0, TAU)) * initial_speed;
        let spin = rng.range_f32(-QIX_MAX_SPIN, QIX_MAX_SPIN);
        Self {
            speed,
            ..Self::new(centre - half, centre + half, vel, spin)
        }
    }

    /// One endpoint of the segment.
    pub fn a(&self) -> Vec2 {
        self.a
    }

    /// The other endpoint of the segment.
    pub fn b(&self) -> Vec2 {
        self.b
    }

    /// The displacement of the midpoint per tick.
    pub fn vel(&self) -> Vec2 {
        self.vel
    }

    /// The turn of the segment per tick, in radians.
    pub fn spin(&self) -> f32 {
        self.spin
    }

    /// The speeds the midpoint keeps within, in playfield units per tick.
    pub fn speed(&self) -> &RangeInclusive<f32> {
        &self.speed
    }

    /// The last segments, oldest first, the current one last; at most
    /// [`QIX_TRAIL_LEN`] of them.
    pub fn trail(&self) -> &VecDeque<(Vec2, Vec2)> {
        &self.trail
    }

    /// The cell under the midpoint: the seed of the area that stays free
    /// when a stix closes.
    pub fn cell(&self) -> (u16, u16) {
        let mid = self.a.midpoint(self.b);
        (mid.x.floor() as u16, mid.y.floor() as u16)
    }

    /// Advances the Qix by one tick on `board`: jitters its velocity and
    /// spin from `rng`, moves and turns it, bouncing off claimed cells and
    /// the board's edges, and records the new segment in the trail.
    pub fn tick(&mut self, board: &Board, rng: &mut Rng) {
        self.jitter(rng);
        self.advance(board);
        if self.trail.len() == QIX_TRAIL_LEN {
            self.trail.pop_front();
        }
        self.trail.push_back((self.a, self.b));
    }

    /// Kicks the velocity and the spin by random amounts, keeping the
    /// speed within [`Qix::speed`] and the spin within [`QIX_MAX_SPIN`].
    fn jitter(&mut self, rng: &mut Rng) {
        let kick = Vec2::new(
            rng.range_f32(-QIX_JITTER, QIX_JITTER),
            rng.range_f32(-QIX_JITTER, QIX_JITTER),
        );
        self.vel = clamp_speed(self.vel + kick, &self.speed);
        let spin = self.spin + rng.range_f32(-QIX_SPIN_JITTER, QIX_SPIN_JITTER);
        self.spin = spin.clamp(-QIX_MAX_SPIN, QIX_MAX_SPIN);
    }

    /// Moves by the velocity and turns by the spin when the segment lands
    /// on free cells only. Otherwise it reflects: the first of the
    /// velocity mirrored on `x`, on `y`, on both, then the same with the
    /// spin reversed, that lands on free cells is taken and kept. When
    /// none does, the Qix stays put and reverses both for the next tick.
    fn advance(&mut self, board: &Board) {
        let Vec2 { x, y } = self.vel;
        let mirrors = [
            Vec2::new(x, y),
            Vec2::new(-x, y),
            Vec2::new(x, -y),
            Vec2::new(-x, -y),
        ];
        let landing = [self.spin, -self.spin]
            .into_iter()
            .flat_map(|spin| mirrors.map(|vel| (vel, spin)))
            .map(|(vel, spin)| (vel, spin, self.moved(vel, spin)))
            .find(|&(_, _, (a, b))| is_free(board, a, b));
        match landing {
            Some((vel, spin, (a, b))) => {
                (self.vel, self.spin) = (vel, spin);
                (self.a, self.b) = (a, b);
            }
            None => (self.vel, self.spin) = (-self.vel, -self.spin),
        }
    }

    /// The segment moved by `vel` and turned by `spin` about its midpoint.
    fn moved(&self, vel: Vec2, spin: f32) -> (Vec2, Vec2) {
        let mid = self.a.midpoint(self.b) + vel;
        let half = (self.b - self.a)
            .rotated(spin)
            .with_length(self.length * 0.5);
        (mid - half, mid + half)
    }
}

/// `vel` scaled to the nearest speed within `bounds`; a standstill
/// becomes the slowest speed along `x`.
fn clamp_speed(vel: Vec2, bounds: &RangeInclusive<f32>) -> Vec2 {
    let speed = vel.length();
    if speed > 0.0 {
        vel * (speed.clamp(*bounds.start(), *bounds.end()) / speed)
    } else {
        Vec2::new(*bounds.start(), 0.0)
    }
}

/// Every cell the segment from `a` to `b` passes through is a free cell of
/// `board`; cells off the board read as claimed.
fn is_free(board: &Board, a: Vec2, b: Vec2) -> bool {
    segment::cells(a, b).all(|(x, y)| !board.cell(x, y).is_claimed())
}
