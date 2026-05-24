//! Real-valued intervals `[min, max]`.
//!
//! Used to bound the valid `t` parameter range during ray-geometry
//! intersection. [`Interval::contains`] is inclusive on both ends;
//! [`Interval::surrounds`] is strictly exclusive — the latter is what you
//! want to reject grazing hits at a boundary (e.g. self-intersection at
//! the surface a ray just left).

/// A closed range `[min, max]` on the real number line.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Interval {
    pub min: f64,
    pub max: f64,
}

impl Interval {
    /// Constructs an interval from its endpoints. No validation — empty
    /// (`min > max`) and degenerate intervals are allowed.
    pub const fn new(min: f64, max: f64) -> Self {
        Self { min, max }
    }

    /// Returns `max - min`. Negative for empty intervals.
    pub fn size(&self) -> f64 {
        self.max - self.min
    }

    /// Returns `true` if `min ≤ x ≤ max`.
    pub fn contains(&self, x: f64) -> bool {
        self.min <= x && x <= self.max
    }

    /// Returns `true` if `min < x < max` (excludes endpoints).
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

    /// Empty interval. `min` is `+∞`, `max` is `-∞` so unions widen correctly.
    pub const EMPTY: Self = Self {
        min: f64::INFINITY,
        max: f64::NEG_INFINITY,
    };

    /// Interval containing every finite real number.
    pub const UNIVERSE: Self = Self {
        min: f64::NEG_INFINITY,
        max: f64::INFINITY,
    };
}

impl Default for Interval {
    fn default() -> Self {
        Self {
            min: f64::INFINITY,
            max: f64::NEG_INFINITY,
        }
    }
}
