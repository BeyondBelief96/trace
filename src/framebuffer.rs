//! In-memory image buffer and the encoders that serialize it to disk.
//!
//! The renderer produces a [`Framebuffer`] — a flat row-major grid of
//! linear-RGB [`Color`] samples — independent of any file format. From
//! there, [`write_ppm`] and [`write_png`] each take that same buffer and
//! emit a file in their respective format, so a single render produces
//! both outputs without re-tracing any rays.
//!
//! Both encoders go through [`color::color_to_rgb8`], so clamp + gamma
//! policy lives in exactly one place.

use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::path::Path;

use crate::color::{Color, color_to_rgb8};

/// Row-major framebuffer of linear-RGB samples. `pixels[j * width + i]`
/// is pixel `(i, j)`, with `(0, 0)` at the top-left of the image.
pub struct Framebuffer {
    pub width: u32,
    pub height: u32,
    pixels: Vec<Color>,
}

impl Framebuffer {
    /// Allocates a black framebuffer of the given dimensions.
    pub fn new(width: u32, height: u32) -> Self {
        let len = (width as usize) * (height as usize);
        Self {
            width,
            height,
            pixels: vec![Color::default(); len],
        }
    }

    /// Stores `color` at pixel `(i, j)`. Panics on out-of-bounds indices.
    pub fn set_pixel(&mut self, i: u32, j: u32, color: Color) {
        let idx = (j as usize) * (self.width as usize) + (i as usize);
        self.pixels[idx] = color;
    }

    /// Flattens the buffer into a `[r, g, b, r, g, b, ...]` byte vector
    /// suitable for handing to PNG / BMP / etc. encoders. Applies the
    /// shared clamp + gamma policy via [`color_to_rgb8`].
    fn to_rgb8_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(self.pixels.len() * 3);
        for &pixel in &self.pixels {
            bytes.extend_from_slice(&color_to_rgb8(pixel));
        }
        bytes
    }
}

/// Writes `framebuffer` to `path` as a PPM (P3) image. Kept around
/// alongside PNG because it's text-based, trivial to diff, and useful
/// when debugging the encoder itself.
pub fn write_ppm(framebuffer: &Framebuffer, path: &Path) -> io::Result<()> {
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);
    writeln!(writer, "P3")?;
    writeln!(writer, "{} {}", framebuffer.width, framebuffer.height)?;
    writeln!(writer, "255")?;
    for &pixel in &framebuffer.pixels {
        let [r, g, b] = color_to_rgb8(pixel);
        writeln!(writer, "{r} {g} {b}")?;
    }
    writer.flush()
}

/// Writes `framebuffer` to `path` as a PNG image.
///
/// Errors from the `image` crate (encoding failures, unsupported dimensions,
/// I/O failure) are converted into an `io::Error` so callers can handle a
/// single error type across both encoders.
pub fn write_png(framebuffer: &Framebuffer, path: &Path) -> io::Result<()> {
    let bytes = framebuffer.to_rgb8_bytes();
    image::save_buffer(
        path,
        &bytes,
        framebuffer.width,
        framebuffer.height,
        image::ColorType::Rgb8,
    )
    .map_err(io::Error::other)
}
