//! Three-dimensional vector math.
//!
//! Provides [`Vec3`], a 3D vector of `f64` components used throughout the
//! renderer for positions, directions, offsets, and colors. The [`Point3`]
//! alias exists purely for readability when a value represents a location
//! rather than a free direction.
//!
//! Standard arithmetic operators are implemented:
//! - vector ± vector, with `+=` / `-=` counterparts
//! - vector × scalar (and scalar × vector), with `*=` / `/=` counterparts
//! - vector × vector as the Hadamard (component-wise) product
//! - unary negation, indexing by `0`/`1`/`2`, and a `Display` impl that
//!   prints components space-separated.

use std::ops::{
    Add, AddAssign, Div, DivAssign, Index, IndexMut, Mul, MulAssign, Neg, Sub, SubAssign,
};

use std::fmt::Display;

/// Alias for [`Vec3`] used when the value represents a point in space rather
/// than a direction. Purely semantic — they share the same layout and methods.
pub type Point3 = Vec3;

/// A 3D vector with `f64` components.
///
/// Used for positions, directions, displacements, and colors. Component
/// access is available through the public fields or via `Index<usize>`
/// (`0 → x`, `1 → y`, `2 → z`).
#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vec3 {
    /// Constructs a vector from its three components.
    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    /// Returns the dot product `self · v`.
    ///
    /// Equals `|self| * |v| * cos(θ)`, where `θ` is the angle between the
    /// vectors. Useful for projection, orthogonality tests (zero result),
    /// and angle comparisons without taking a square root.
    pub fn dot(&self, v: Self) -> f64 {
        self.x * v.x + self.y * v.y + self.z * v.z
    }

    /// Returns the cross product `self × v`.
    ///
    /// The result is perpendicular to both inputs, with a length equal to
    /// the area of the parallelogram they span. The orientation follows the
    /// right-hand rule.
    pub fn cross(&self, v: Self) -> Self {
        Self::new(
            self.y * v.z - self.z * v.y,
            self.z * v.x - self.x * v.z,
            self.x * v.y - self.y * v.x,
        )
    }

    /// Returns `self` scaled to unit length (i.e. `self / |self|`).
    ///
    /// Produces non-finite components if `self` has zero length; callers are
    /// expected to avoid normalizing the zero vector.
    pub fn unit_vector(&self) -> Self {
        *self / self.length()
    }

    /// Returns the Euclidean length `sqrt(x² + y² + z²)`.
    pub fn length(&self) -> f64 {
        self.length_squared().sqrt()
    }

    /// Returns the squared length `x² + y² + z²`.
    ///
    /// Prefer this over [`Vec3::length`] when comparing magnitudes — it
    /// avoids a square root, and for many tests (e.g. "is this inside the
    /// unit ball?") the squared form is what you actually want.
    pub fn length_squared(&self) -> f64 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    /// Returns a vector with each component independently sampled from
    /// `[0, 1)` via a uniform distribution.
    ///
    /// Because every component is non-negative, the result always lies in
    /// the positive octant (`x ≥ 0, y ≥ 0, z ≥ 0`) and is **not**
    /// normalized — its length ranges from `0` up to `√3`.
    pub fn random() -> Self {
        Self::new(
            rand::random::<f64>(),
            rand::random::<f64>(),
            rand::random::<f64>(),
        )
    }

    /// Returns a vector with each component independently sampled from
    /// `[min, max)` via a uniform distribution.
    pub fn random_in_range(min: f64, max: f64) -> Self {
        Self::new(
            rand::random::<f64>() * (max - min) + min,
            rand::random::<f64>() * (max - min) + min,
            rand::random::<f64>() * (max - min) + min,
        )
    }

    /// Returns a unit vector pointing in a uniformly random direction on
    /// the sphere.
    ///
    /// Implemented via **rejection sampling**: a candidate is drawn
    /// uniformly from the cube `[-1, 1]³`, discarded if it falls outside
    /// the unit ball, and otherwise normalized.
    ///
    /// Normalizing a cube sample directly would bias directions toward the
    /// cube's eight corners, since the corners extend farther from the
    /// origin than the face centers and therefore project a larger volume
    /// onto those corner directions. Sampling from the (rotationally
    /// symmetric) unit ball removes that bias.
    ///
    /// Roughly 48% of candidates are rejected — the ball occupies
    /// `π/6 ≈ 52%` of the enclosing cube's volume — so the loop terminates
    /// effectively immediately.
    pub fn random_unit_vector() -> Self {
        loop {
            let p = Vec3::random_in_range(-1.0, 1.0);
            if 1e-160 < p.length_squared() && p.length_squared() <= 1.0 {
                return p.unit_vector();
            }
        }
    }

    /// Returns a random unit vector on the hemisphere with the given normal.
    pub fn random_on_hemisphere(normal: &Vec3) -> Self {
        let on_unit_sphere = Vec3::random_unit_vector();
        if on_unit_sphere.dot(*normal) > 0.0 {
            on_unit_sphere
        } else {
            -on_unit_sphere
        }
    }

    /// Returns `true` if the vector is near zero, i.e. all components are within `1e-8` of zero.
    pub fn near_zero(&self) -> bool {
        self.x.abs() < 1e-8 && self.y.abs() < 1e-8 && self.z.abs() < 1e-8
    }

    /// Returns the reflection of this vector about the given normal.
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

/// Component-wise (Hadamard) product: `(a.x*b.x, a.y*b.y, a.z*b.z)`.
///
/// Note this is **not** the dot or cross product. It is most often used
/// for attenuating one color by another (e.g. surface albedo × incoming
/// light).
impl Mul<Vec3> for Vec3 {
    type Output = Vec3;

    fn mul(self, rhs: Vec3) -> Self::Output {
        Vec3::new(self.x * rhs.x, self.y * rhs.y, self.z * rhs.z)
    }
}

/// Scalar broadcast on the left: `s * v` scales every component of `v` by `s`.
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

/// Scalar broadcast on the left: `s * v` scales every component of `v` by `s`.
impl Mul<Vec3> for f64 {
    type Output = Vec3;

    fn mul(self, rhs: Vec3) -> Self::Output {
        Vec3::new(self * rhs.x, self * rhs.y, self * rhs.z)
    }
}

/// Scalar broadcast on the right: `v * s` scales every component of `v` by `s`.
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

/// Indexes components by axis: `0 → x`, `1 → y`, `2 → z`.
///
/// # Panics
/// Panics if `index` is not `0`, `1`, or `2`.
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

/// Mutable component access by axis: `0 → x`, `1 → y`, `2 → z`.
///
/// # Panics
/// Panics if `index` is not `0`, `1`, or `2`.
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

/// Formats the vector as three space-separated components: `"x y z"`.
impl Display for Vec3 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {} {}", self.x, self.y, self.z)
    }
}
