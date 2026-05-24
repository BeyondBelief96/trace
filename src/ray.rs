//! Parametric rays for tracing geometry queries.
//!
//! A [`Ray`] is a half-line defined by an origin point and a direction
//! vector, evaluated at a scalar parameter `t` via
//! `point(t) = origin + t * direction`. Convention here is:
//! - `t = 0` is the ray's origin
//! - increasing `t` moves along `direction`
//! - `t < 0` is "behind" the ray, and intersection routines reject it

use crate::vec3::Vec3;

/// A half-line in 3D, parameterized by a scalar `t ≥ 0`.
///
/// `direction` is not required to be unit-length. Keeping it un-normalized
/// is convenient (e.g. for rays cast through pixel offsets), but consumers
/// that need physical distance — rather than the parameter `t` — must
/// account for `|direction|`.
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

    /// Returns the point at parameter `t` along the ray:
    /// `origin + t * direction`.
    pub fn at(&self, t: f64) -> Vec3 {
        self.origin + t * self.direction
    }
}
