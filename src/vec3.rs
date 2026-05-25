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
///
/// Same underlying type — the alias only signals intent. Because it's an
/// alias and not a newtype, the compiler will not catch nonsensical
/// operations like adding two points or dotting a position with a
/// direction. Upgrade to a newtype if that becomes a real source of bugs.
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
    ///
    /// The `1e-160` lower bound on `length_squared` rejects denormal-tiny
    /// samples whose square root would underflow to zero, making the
    /// subsequent normalization divide by zero and return infinities.
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

    /// Uniformly random point inside the unit disk in the xy plane (`z = 0`).
    ///
    /// Rejection sampling — same idea as [`random_unit_vector`], one
    /// dimension lower. Used to pick lens-disk origins for defocus blur.
    /// Returns an *interior* point (no normalization); the caller scales
    /// it by the disk's basis vectors to land on the actual lens.
    ///
    /// [`random_unit_vector`]: Vec3::random_unit_vector
    pub fn random_in_unit_disk() -> Self {
        loop {
            let p = Vec3::new(
                rand::random_range(-1.0..1.0),
                rand::random_range(-1.0..1.0),
                0.0,
            );
            if p.length_squared() < 1.0 {
                return p;
            }
        }
    }

    /// Returns `true` if every component is within `1e-8` of zero.
    ///
    /// Use when a vector is about to be normalized — `unit_vector()` on
    /// something this small produces garbage from floating-point noise.
    pub fn near_zero(&self) -> bool {
        self.x.abs() < 1e-8 && self.y.abs() < 1e-8 && self.z.abs() < 1e-8
    }

    /// Reflects `self` about `normal`.
    ///
    /// Convention: `self` is the incident direction pointing *into* the
    /// surface, and `normal` points *out* of the surface (same side as the
    /// incoming ray). Both are assumed to be unit length — the formula
    /// `R - 2(R·n)n` only produces a unit-length result when they are.
    pub fn reflect(&self, normal: &Vec3) -> Self {
        *self - 2.0 * self.dot(*normal) * *normal
    }

    /// Refracts `self` through a surface using Snell's law.
    ///
    /// `etai_over_etat` is the **ratio** η/η' — incident medium's index of
    /// refraction divided by the transmitted medium's — not a single
    /// material's IOR. For a ray entering glass (η'=1.5) from air (η=1.0),
    /// pass `1.0 / 1.5`; for the reverse, pass `1.5 / 1.0`.
    ///
    /// Convention: `self` is the incident direction pointing *into* the
    /// surface, and `normal` points *out* of the surface (same side as the
    /// incoming ray). Both `self` and `normal` must be unit length — the
    /// derivation cancels their magnitudes, so non-unit inputs produce a
    /// silently wrong result.
    ///
    /// Does **not** handle total internal reflection. When
    /// `etai_over_etat * sin(θ) > 1` refraction is physically impossible
    /// and the inner `sqrt` produces `NaN`. The caller (typically a
    /// dielectric material) must detect that case and reflect instead.
    pub fn refract(&self, normal: &Vec3, etai_over_etat: f64) -> Self {
        let cos_theta = (-1.0 * self.dot(*normal)).min(1.0);
        let r_out_perp = etai_over_etat * (*self + cos_theta * *normal);
        let r_out_parallel = -1.0 * (1.0 - r_out_perp.length_squared()).sqrt();
        r_out_perp + r_out_parallel * *normal
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

/// Scalar–vector multiplication. The mirror impls below let you write the
/// scalar on either side: both `2.0 * v` and `v * 2.0` compile.
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
