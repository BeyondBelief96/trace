//! Three-dimensional vector math.
//!
//! Provides [`Vec3`], a 3D vector of `f64` components used throughout the
//! renderer for positions, directions, offsets, and colors. The [`Point3`]
//! alias is purely semantic — same type, signals "location" rather than
//! "free direction."
//!
//! Standard arithmetic operators are implemented, including the Hadamard
//! (component-wise) product `Vec3 * Vec3`, which is the natural operation
//! for attenuating one color by another.

use std::ops::{
    Add, AddAssign, Div, DivAssign, Index, IndexMut, Mul, MulAssign, Neg, Sub, SubAssign,
};

use std::fmt::Display;

/// Alias for [`Vec3`] used for positions rather than directions.
pub type Point3 = Vec3;

/// A 3D vector with `f64` components.
#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vec3 {
    /// Constructs a vector from its components.
    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    /// Returns the dot product `self · v` = `|self| |v| cos(θ)`.
    pub fn dot(&self, v: Self) -> f64 {
        self.x * v.x + self.y * v.y + self.z * v.z
    }

    /// Returns the cross product `self × v` (right-hand rule).
    pub fn cross(&self, v: Self) -> Self {
        Self::new(
            self.y * v.z - self.z * v.y,
            self.z * v.x - self.x * v.z,
            self.x * v.y - self.y * v.x,
        )
    }

    /// Returns `self` scaled to unit length. Non-finite if `self` is zero.
    pub fn unit_vector(&self) -> Self {
        *self / self.length()
    }

    /// Returns the Euclidean length `sqrt(x² + y² + z²)`.
    pub fn length(&self) -> f64 {
        self.length_squared().sqrt()
    }

    /// Returns `x² + y² + z²`. Prefer this over [`Vec3::length`] when
    /// comparing magnitudes — avoids the square root.
    pub fn length_squared(&self) -> f64 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    /// Each component sampled uniformly from `[0, 1)`. Lies in the positive
    /// octant; not normalized.
    pub fn random() -> Self {
        Self::new(
            rand::random::<f64>(),
            rand::random::<f64>(),
            rand::random::<f64>(),
        )
    }

    /// Each component sampled uniformly from `[min, max)`.
    pub fn random_in_range(min: f64, max: f64) -> Self {
        Self::new(
            rand::random::<f64>() * (max - min) + min,
            rand::random::<f64>() * (max - min) + min,
            rand::random::<f64>() * (max - min) + min,
        )
    }

    /// Uniformly random unit vector on the sphere.
    ///
    /// Uses rejection sampling — draw from the cube `[-1, 1]³`, keep only
    /// candidates inside the unit ball, then normalize. Normalizing a raw
    /// cube sample would bias directions toward the cube's eight corners;
    /// sampling from the (rotationally symmetric) ball removes that bias.
    /// Rejection rate is ~48% (`1 - π/6`), so the loop is effectively O(1).
    pub fn random_unit_vector() -> Self {
        loop {
            let p = Vec3::random_in_range(-1.0, 1.0);
            if 1e-160 < p.length_squared() && p.length_squared() <= 1.0 {
                return p.unit_vector();
            }
        }
    }

    /// Uniformly random unit vector on the hemisphere defined by `normal`.
    pub fn random_on_hemisphere(normal: &Vec3) -> Self {
        let on_unit_sphere = Vec3::random_unit_vector();
        if on_unit_sphere.dot(*normal) > 0.0 {
            on_unit_sphere
        } else {
            -on_unit_sphere
        }
    }

    /// Returns `true` if every component is within `1e-8` of zero.
    ///
    /// Use when a vector is about to be normalized — `unit_vector()` on
    /// something this small produces garbage from floating-point noise.
    pub fn near_zero(&self) -> bool {
        self.x.abs() < 1e-8 && self.y.abs() < 1e-8 && self.z.abs() < 1e-8
    }

    /// Reflects `self` about `normal`. `normal` is assumed unit length.
    pub fn reflect(&self, normal: &Vec3) -> Self {
        *self - 2.0 * self.dot(*normal) * *normal
    }
}

impl Add for Vec3 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl AddAssign for Vec3 {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

impl Sub for Vec3 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl SubAssign for Vec3 {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
    }
}

/// Hadamard (component-wise) product — **not** dot or cross. Used for
/// attenuating one color by another.
impl Mul<Vec3> for Vec3 {
    type Output = Vec3;

    fn mul(self, rhs: Vec3) -> Self::Output {
        Vec3::new(self.x * rhs.x, self.y * rhs.y, self.z * rhs.z)
    }
}

impl Mul<Vec3> for i32 {
    type Output = Vec3;

    fn mul(self, rhs: Vec3) -> Self::Output {
        Vec3::new(
            f64::from(self) * rhs.x,
            f64::from(self) * rhs.y,
            f64::from(self) * rhs.z,
        )
    }
}

impl Mul<Vec3> for f64 {
    type Output = Vec3;

    fn mul(self, rhs: Vec3) -> Self::Output {
        Vec3::new(self * rhs.x, self * rhs.y, self * rhs.z)
    }
}

impl Mul<f64> for Vec3 {
    type Output = Self;

    fn mul(self, rhs: f64) -> Self::Output {
        Vec3::new(self.x * rhs, self.y * rhs, self.z * rhs)
    }
}

impl MulAssign<f64> for Vec3 {
    fn mul_assign(&mut self, rhs: f64) {
        self.x *= rhs;
        self.y *= rhs;
        self.z *= rhs;
    }
}

impl DivAssign<f64> for Vec3 {
    fn div_assign(&mut self, rhs: f64) {
        self.x /= rhs;
        self.y /= rhs;
        self.z /= rhs;
    }
}

impl Div<f64> for Vec3 {
    type Output = Self;

    fn div(self, rhs: f64) -> Self::Output {
        Vec3::new(self.x / rhs, self.y / rhs, self.z / rhs)
    }
}

impl Neg for Vec3 {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Vec3::new(-self.x, -self.y, -self.z)
    }
}

/// Component access by axis: `0 → x`, `1 → y`, `2 → z`. Panics otherwise.
impl Index<usize> for Vec3 {
    type Output = f64;

    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.x,
            1 => &self.y,
            2 => &self.z,
            _ => panic!("Index out of bounds"),
        }
    }
}

/// Mutable component access by axis: `0 → x`, `1 → y`, `2 → z`. Panics otherwise.
impl IndexMut<usize> for Vec3 {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match index {
            0 => &mut self.x,
            1 => &mut self.y,
            2 => &mut self.z,
            _ => panic!("Index out of bounds"),
        }
    }
}

/// Formats as `"x y z"` (space-separated).
impl Display for Vec3 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {} {}", self.x, self.y, self.z)
    }
}
