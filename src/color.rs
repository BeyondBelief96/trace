//! RGB colors and the linear → 8-bit conversion shared by every encoder.
//!
//! [`Color`] aliases [`Vec3`], so vector arithmetic applies directly:
//! addition for accumulating samples, scalar multiplication for averaging,
//! Hadamard product for attenuation. Channels map `x → R`, `y → G`,
//! `z → B`. Values are linear and unbounded; [`color_to_rgb8`] clamps and
//! gamma-corrects them on the way out to whichever image format we're
//! producing (see `framebuffer.rs`).

use crate::{interval::Interval, vec3::Vec3};

/// Linear RGB color (`x → R`, `y → G`, `z → B`). Not gamma-corrected.
pub type Color = Vec3;

/// Gamma-2 correction: `sqrt(linear)`. Negative inputs map to zero.
pub fn linear_to_gamma(linear_component: f64) -> f64 {
    if linear_component <= 0.0 {
        return 0.0;
    }

    linear_component.sqrt()
}

/// Converts a single linear channel to an 8-bit gamma-corrected byte.
///
/// Clamps to `[0, 0.999]`, applies gamma-2 correction, then scales by 256
/// and truncates. The half-open upper bound prevents the scaled value
/// from ever reaching 256 and overflowing the `u8` cast.
#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::cast_sign_loss)]
fn channel_to_byte(linear: f64) -> u8 {
    let intensity = Interval::new(0.000, 0.999);
    let clamped = intensity.clamp(linear);
    let corrected = linear_to_gamma(clamped);
    (corrected * 256.0) as u8
}

/// Converts a linear `Color` to a gamma-corrected `[r, g, b]` byte triple.
///
/// Shared by every output encoder so the gamma and clamp policy lives in
/// exactly one place.
pub fn color_to_rgb8(pixel_color: Color) -> [u8; 3] {
    [
        channel_to_byte(pixel_color.x),
        channel_to_byte(pixel_color.y),
        channel_to_byte(pixel_color.z),
    ]
}
