//! Camera model and the render loop.
//!
//! A [`Camera`] is constructed through [`CameraBuilder`], which lets you
//! override individual parameters fluently and falls back to sensible
//! defaults for the rest. The camera then owns the [`Camera::render`]
//! method that walks the image plane pixel-by-pixel, casts rays into a
//! scene, accumulates samples, and writes the result as a PPM (P3) image.
//!
//! ### Coordinate setup
//!
//! - The camera sits at the origin and looks down the `-z` axis.
//! - The viewport (the rectangle of world-space the image plane samples
//!   through) sits at `z = -focal_length`.
//! - Image rows count top-to-bottom; the viewport's `v` axis therefore
//!   points in `-y` so that increasing pixel index `j` corresponds to
//!   moving down the image.
//!
//! ### Anti-aliasing
//!
//! Each output pixel averages `samples_per_pixel` rays, with each ray
//! jittered by a uniform offset inside the pixel's square footprint. This
//! is plain box-filter supersampling — cheap, and enough to smooth the
//! hard staircase you'd get from a single ray per pixel.

use std::io::{self, Write};

use crate::{
    color::{Color, write_color},
    hittable::Hittable,
    interval::Interval,
    ray::Ray,
    vec3::{Point3, Vec3},
};

/// Fluent builder for [`Camera`]. Each setter consumes and returns `Self`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CameraBuilder {
    pub aspect_ratio: f64,
    pub image_width: u32,
    pub samples_per_pixel: u32,
    pub maximum_depth: u32,
}

impl CameraBuilder {
    /// Builder pre-populated with default settings.
    pub fn new() -> Self {
        Self {
            aspect_ratio: 16.0 / 9.0,
            image_width: 400,
            samples_per_pixel: 10,
            maximum_depth: 10,
        }
    }

    pub const fn aspect_ratio(mut self, aspect_ratio: f64) -> Self {
        self.aspect_ratio = aspect_ratio;
        self
    }

    pub const fn image_width(mut self, image_width: u32) -> Self {
        self.image_width = image_width;
        self
    }

    pub const fn samples_per_pixel(mut self, samples_per_pixel: u32) -> Self {
        self.samples_per_pixel = samples_per_pixel;
        self
    }

    pub const fn maximum_depth(mut self, maximum_depth: u32) -> Self {
        self.maximum_depth = maximum_depth;
        self
    }

    /// Computes viewport geometry and returns a ready [`Camera`].
    ///
    /// Image height is `image_width / aspect_ratio`, floored at 1 (very
    /// wide ratios can otherwise truncate to zero). The viewport is sized
    /// from the *actual* integer image dimensions so pixels stay square
    /// after rounding.
    pub fn build(self) -> Camera {
        #[allow(clippy::cast_possible_truncation)]
        #[allow(clippy::cast_sign_loss)]
        let image_height = (f64::from(self.image_width) / self.aspect_ratio) as u32;
        let image_height = if image_height < 1 { 1 } else { image_height };

        // Viewport: a 2-unit-tall window sitting `focal_length` in front of
        // the camera, with its width chosen to match the integer aspect
        // ratio of the final image.
        let focal_length = 1.0;
        let viewport_height = 2.0;
        let viewport_width =
            viewport_height * (f64::from(self.image_width) / f64::from(image_height));
        let camera_center = Point3::new(0.0, 0.0, 0.0);

        // Edge vectors along the viewport. `viewport_v` points in `-y`
        // because pixel rows are written top-to-bottom.
        let viewport_u = Vec3::new(viewport_width, 0.0, 0.0);
        let viewport_v = Vec3::new(0.0, -viewport_height, 0.0);

        // Per-pixel spacing in world units.
        let pixel_delta_u = viewport_u / f64::from(self.image_width);
        let pixel_delta_v = viewport_v / f64::from(image_height);

        // Place pixel (0, 0) half a pixel in from the viewport's top-left
        // corner, so each pixel's center — not its corner — lies on the grid.
        let viewport_upper_left = camera_center
            - Vec3::new(0.0, 0.0, focal_length)
            - (viewport_u / 2.0)
            - (viewport_v / 2.0);
        let pixel00_location = viewport_upper_left + 0.5 * (pixel_delta_u + pixel_delta_v);

        Camera {
            image_width: self.image_width,
            image_height,
            center: camera_center,
            pixel00_location,
            pixel_delta_u,
            pixel_delta_v,
            samples_per_pixel: self.samples_per_pixel,
            pixel_samples_scale: 1.0 / f64::from(self.samples_per_pixel),
            maximum_depth: self.maximum_depth,
        }
    }
}

impl Default for CameraBuilder {
    fn default() -> Self {
        Self {
            aspect_ratio: 16.0 / 9.0,
            image_width: 400,
            samples_per_pixel: 10,
            maximum_depth: 10,
        }
    }
}

/// A configured camera ready to render a scene. Construct via
/// [`CameraBuilder`]; fields are private because they must satisfy
/// invariants the builder enforces (matching deltas, derived height,
/// precomputed sample scale).
pub struct Camera {
    image_width: u32,
    image_height: u32,
    center: Point3,
    /// World-space center of pixel (0, 0).
    pixel00_location: Point3,
    /// World-space step between adjacent pixels in a row / column.
    pixel_delta_u: Vec3,
    pixel_delta_v: Vec3,
    samples_per_pixel: u32,
    /// `1 / samples_per_pixel`, cached for averaging the per-pixel sum.
    pixel_samples_scale: f64,
    maximum_depth: u32,
}

impl Camera {
    /// Renders `world` to `writer` as a PPM (P3) image. Progress is logged
    /// to stderr a scanline at a time.
    pub fn render<W: Write>(&self, world: &dyn Hittable, writer: &mut W) -> io::Result<()> {
        writeln!(writer, "P3")?;
        writeln!(writer, "{} {}", self.image_width, self.image_height)?;
        writeln!(writer, "255")?;
        for j in 0..self.image_height {
            eprintln!("Scanlines remaining: {}", self.image_height - j);
            for i in 0..self.image_width {
                let mut pixel_color = Color::default();
                for _ in 0..self.samples_per_pixel {
                    let ray = self.get_ray(i, j);
                    pixel_color += Self::ray_color(&ray, world, self.maximum_depth)
                }
                write_color(writer, pixel_color * self.pixel_samples_scale)?;
            }
        }

        writer.flush()?;
        println!("\n Done!");
        Ok(())
    }

    /// Recursively traces `r`, returning the color it carries back.
    ///
    /// On a hit, the surface's material decides how to scatter; the
    /// scattered ray's color is multiplied by the attenuation. On a miss,
    /// returns a vertical sky gradient (white → pale blue). Bottoms out
    /// at black when `depth` reaches zero or scattering fails.
    fn ray_color(r: &Ray, world: &dyn Hittable, depth: u32) -> Color {
        if depth <= 0 {
            return Color::new(0.0, 0.0, 0.0);
        }

        // 0.001 is to avoid shadow acne.
        if let Some(rec) = world.hit(r, Interval::new(0.001, f64::INFINITY)) {
            let scattered = rec.material.scatter(&r, &rec);
            if let Some(scattered) = scattered {
                return scattered.attenuation * Self::ray_color(&scattered.ray, world, depth - 1);
            }

            return Color::new(0.0, 0.0, 0.0);
        }

        let unit_direction = r.direction.unit_vector();
        let a = 0.5 * (unit_direction.y + 1.0);
        (1.0 - a) * Color::new(1.0, 1.0, 1.0) + a * Color::new(0.5, 0.7, 1.0)
    }

    /// Camera ray through pixel `(i, j)`, jittered for anti-aliasing.
    fn get_ray(&self, i: u32, j: u32) -> Ray {
        let offset = Self::sample_square();
        let pixel_sample = self.pixel00_location
            + (f64::from(i) + offset.x) * self.pixel_delta_u
            + (f64::from(j) + offset.y) * self.pixel_delta_v;
        let ray_origin = self.center;
        let ray_direction = pixel_sample - self.center;
        Ray::new(ray_origin, ray_direction)
    }

    /// Uniform random offset in `[-0.5, 0.5)² × {0}`.
    fn sample_square() -> Point3 {
        Point3::new(
            rand::random::<f64>() - 0.5,
            rand::random::<f64>() - 0.5,
            0.0,
        )
    }
}
