//! Parametric rays for geometry queries.
//!
//! A [`Ray`] is the half-line `point(t) = origin + t * direction`. By
//! convention `t = 0` is the origin, increasing `t` moves along the
//! direction, and intersection routines reject `t < 0` ("behind" the ray).

use crate::vec3::Vec3;

/// A half-line `origin + t * direction`. `direction` is not required to
/// be unit length — callers needing physical distance must account for
/// `|direction|`.
#[derive(Debug, Clone, Copy)]
pub struct Ray {
    pub origin: Vec3,
    pub direction: Vec3,
}

impl Ray {
    /// Constructs a ray from its origin and direction.
    pub const fn new(origin: Vec3, direction: Vec3) -> Self {
        Self { origin, direction }
    }

    /// Returns `origin + t * direction`.
    pub fn at(&self, t: f64) -> Vec3 {
        self.origin + t * self.direction
    }
}
