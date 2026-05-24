//! Real-valued intervals on the number line.
//!
//! [`Interval`] represents a range `[min, max]` and is used to bound the
//! valid `t` parameter range when intersecting rays with geometry. Two
//! membership tests are provided that differ in their endpoint handling:
//! [`Interval::contains`] is inclusive on both ends, while
//! [`Interval::surrounds`] is strictly exclusive — useful when you want
//! to exclude grazing hits exactly at a boundary.

/// A closed range `[min, max]` on the real number line.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Interval {
    pub min: f64,
    pub max: f64,
}

impl Interval {
    /// Constructs an interval from its endpoints.
    ///
    /// No validation is performed: callers may construct empty (`min > max`)
    /// or degenerate (`min == max`) intervals deliberately.
    pub const fn new(min: f64, max: f64) -> Self {
        Self { min, max }
    }

    /// Returns the interval's width, `max - min`.
    ///
    /// Negative for empty intervals.
    pub fn size(&self) -> f64 {
        self.max - self.min
    }

    /// Inclusive membership: returns `true` if `min ≤ x ≤ max`.
    pub fn contains(&self, x: f64) -> bool {
        self.min <= x && x <= self.max
    }

    /// Strict membership: returns `true` if `min < x < max`.
    ///
    /// Preferred over [`Interval::contains`] when boundary hits should be
    /// rejected — for example, when filtering ray-intersection `t` values
    /// to avoid self-intersection at the surface a ray just left.
    pub fn surrounds(&self, x: f64) -> bool {
        self.min < x && x < self.max
    }

    /// Clamps `x` into `[min, max]`.
    pub fn clamp(&self, x: f64) -> f64 {
        if x < self.min {
            self.min
        } else if x > self.max {
            self.max
        } else {
            x
        }
    }

    /// The empty interval: contains no real number.
    ///
    /// Built so that any union operation will widen it correctly (its `min`
    /// is `+∞` and `max` is `-∞`).
    pub const EMPTY: Self = Self {
        min: f64::INFINITY,
        max: f64::NEG_INFINITY,
    };

    /// The universal interval: contains every finite real number.
    pub const UNIVERSE: Self = Self {
        min: f64::NEG_INFINITY,
        max: f64::INFINITY,
    };
}

/// The default interval is empty (`[+∞, -∞]`), matching [`Interval::EMPTY`].
impl Default for Interval {
    fn default() -> Self {
        Self {
            min: f64::INFINITY,
            max: f64::NEG_INFINITY,
        }
    }
}
