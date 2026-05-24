//! RGB colors and PPM pixel output.
//!
//! [`Color`] aliases [`Vec3`], so vector arithmetic applies directly:
//! addition for accumulating samples, scalar multiplication for averaging,
//! Hadamard product for attenuation. Channels map `x → R`, `y → G`,
//! `z → B`. Values are linear and unbounded until [`write_color`] clamps
//! and gamma-corrects them at output time.

use crate::{interval::Interval, vec3::Vec3};
use std::io::{self, Write};

/// Linear RGB color (`x → R`, `y → G`, `z → B`). Not gamma-corrected.
pub type Color = Vec3;

/// Gamma-2 correction: `sqrt(linear)`. Negative inputs map to zero.
pub fn linear_to_gamma(linear_component: f64) -> f64 {
    if linear_component <= 0.0 {
        return 0.0;
    }

    linear_component.sqrt()
}

/// Writes one pixel as PPM (P3) `"R G B\n"`.
///
/// Each channel is clamped to `[0, 0.999)`, gamma-corrected, then scaled
/// by 256 and truncated to `u8`. The half-open upper bound prevents the
/// scaled value from ever reaching 256 and overflowing the cast.
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
