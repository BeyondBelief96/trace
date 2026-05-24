//! RGB colors and PPM pixel output.
//!
//! [`Color`] is a type alias over [`Vec3`], so the full set of vector
//! arithmetic — addition for accumulating samples, scalar multiplication
//! for averaging, and the Hadamard product for attenuation — applies
//! directly to colors. Component conventions:
//! - `x` → red, `y` → green, `z` → blue
//! - linear, unbounded; values are clamped to `[0, 1)` only at output
//!   time by [`write_color`].

use crate::{interval::Interval, vec3::Vec3};
use std::io::{self, Write};

/// A linear RGB color stored as three `f64` components.
///
/// Aliased to [`Vec3`] so all vector operations are available. Components
/// are not gamma-corrected and are not clamped until conversion to bytes.
pub type Color = Vec3;

pub fn linear_to_gamma(linear_component: f64) -> f64 {
    if linear_component <= 0.0 {
        return 0.0;
    }

    linear_component.sqrt()
}

/// Writes a single pixel to `writer` in PPM (P3) "R G B\n" text format.
///
/// Components are clamped to the half-open intensity range `[0, 0.999]`
/// before being scaled by 256 and truncated to `u8`. The half-open upper
/// bound ensures the multiplication can never produce the value 256,
/// which would overflow when cast to `u8`.
pub fn write_color(writer: &mut impl Write, pixel_color: Color) -> io::Result<()> {
    let intensity = Interval::new(0.000, 0.999);
    let r = intensity.clamp(pixel_color.x);
    let g = intensity.clamp(pixel_color.y);
    let b = intensity.clamp(pixel_color.z);

    let r = linear_to_gamma(r);
    let g = linear_to_gamma(g);
    let b = linear_to_gamma(b);

    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_sign_loss)]
    let rbyte: u8 = (r * 256.0) as u8;
    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_sign_loss)]
    let gbyte: u8 = (g * 256.0) as u8;
    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_sign_loss)]
    let bbyte: u8 = (b * 256.0) as u8;

    writeln!(writer, "{rbyte} {gbyte} {bbyte}")
}
