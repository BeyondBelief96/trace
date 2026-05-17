use crate::vec3::Vec3;
use std::io::{self, Write};

pub type Color = Vec3;

pub fn write_color(out: &mut impl Write, pixel_color: Color) -> io::Result<()> {
    let r = pixel_color.x;
    let g = pixel_color.y;
    let b = pixel_color.z;

    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_sign_loss)]
    let rbyte: u8 = (r * 255.999) as u8;
    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_sign_loss)]
    let gbyte: u8 = (g * 255.999) as u8;
    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_sign_loss)]
    let bbyte: u8 = (b * 255.999) as u8;

    writeln!(out, "{rbyte} {gbyte} {bbyte}")
}
