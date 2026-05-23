use crate::{interval::Interval, vec3::Vec3};
use std::io::{self, Write};

pub type Color = Vec3;

/// Writes the given pixel color to the given output stream.
/// Clamps the pixel color components to the range [0, 1] before writing.
pub fn write_color(writer: &mut impl Write, pixel_color: Color) -> io::Result<()> {
    let intensity = Interval::new(0.000, 0.999);
    let r = intensity.clamp(pixel_color.x);
    let g = intensity.clamp(pixel_color.y);
    let b = intensity.clamp(pixel_color.z);

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
