//! Vectors in continuous playfield units, for the entities that are not
//! bound to the lattice.

use std::ops::{Add, Mul, Neg, Sub};

/// A point or a displacement in playfield units: one unit per cell, so
/// cell `(x, y)` covers `[x, x + 1) x [y, y + 1)`; `y` grows downwards.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    /// The vector `(x, y)`.
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// The unit vector at `angle` radians from the `x` axis, towards `y`.
    pub fn from_angle(angle: f32) -> Self {
        let (sin, cos) = angle.sin_cos();
        Self::new(cos, sin)
    }

    /// The Euclidean length.
    pub fn length(self) -> f32 {
        self.x.hypot(self.y)
    }

    /// This vector scaled to `length`; the zero vector stays zero.
    pub fn with_length(self, length: f32) -> Self {
        let current = self.length();
        if current > 0.0 {
            self * (length / current)
        } else {
            self
        }
    }

    /// The point halfway between `self` and `other`.
    pub fn midpoint(self, other: Self) -> Self {
        (self + other) * 0.5
    }

    /// This vector turned by `angle` radians, from the `x` axis towards
    /// `y`.
    pub fn rotated(self, angle: f32) -> Self {
        let (sin, cos) = angle.sin_cos();
        Self::new(self.x * cos - self.y * sin, self.x * sin + self.y * cos)
    }
}

impl Add for Vec2 {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self::new(self.x + other.x, self.y + other.y)
    }
}

impl Sub for Vec2 {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self::new(self.x - other.x, self.y - other.y)
    }
}

impl Mul<f32> for Vec2 {
    type Output = Self;

    fn mul(self, factor: f32) -> Self {
        Self::new(self.x * factor, self.y * factor)
    }
}

impl Neg for Vec2 {
    type Output = Self;

    fn neg(self) -> Self {
        Self::new(-self.x, -self.y)
    }
}
