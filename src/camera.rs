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

/// Fluent builder for [`Camera`].
///
/// Each setter consumes and returns `Self`, so calls can be chained. Call
/// [`CameraBuilder::build`] to compute the derived viewport geometry and
/// produce a ready-to-use [`Camera`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CameraBuilder {
    /// Ratio of image width to image height.
    pub aspect_ratio: f64,
    /// Output image width in pixels. The height is derived from this and
    /// `aspect_ratio`, with a floor of 1.
    pub image_width: u32,
    /// Number of jittered rays cast per output pixel (anti-aliasing).
    pub samples_per_pixel: u32,
}

impl CameraBuilder {
    /// Creates a builder pre-populated with the default camera settings.
    pub fn new() -> Self {
        Self {
            aspect_ratio: 16.0 / 9.0,
            image_width: 400,
            samples_per_pixel: 10,
        }
    }

    /// Sets the target aspect ratio (width / height).
    pub const fn aspect_ratio(mut self, aspect_ratio: f64) -> Self {
        self.aspect_ratio = aspect_ratio;
        self
    }

    /// Sets the output image width in pixels.
    pub const fn image_width(mut self, image_width: u32) -> Self {
        self.image_width = image_width;
        self
    }

    /// Sets the number of jittered samples taken per pixel.
    pub const fn samples_per_pixel(mut self, samples_per_pixel: u32) -> Self {
        self.samples_per_pixel = samples_per_pixel;
        self
    }

    /// Finalizes the builder, computing viewport geometry and producing
    /// a [`Camera`].
    ///
    /// Image height is derived from `image_width / aspect_ratio` and
    /// clamped up to at least 1, because very wide aspect ratios can
    /// otherwise truncate to zero. Viewport dimensions are then computed
    /// from the *actual* integer image dimensions rather than the target
    /// aspect ratio, so the pixels stay square even after that rounding.
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
        }
    }
}

impl Default for CameraBuilder {
    fn default() -> Self {
        Self {
            aspect_ratio: 16.0 / 9.0,
            image_width: 400,
            samples_per_pixel: 10,
        }
    }
}

/// A configured camera ready to render a scene.
///
/// Construct one via [`CameraBuilder`]; the fields are private because
/// they have to satisfy invariants (matching deltas, derived height,
/// precomputed sample scale) that the builder enforces.
pub struct Camera {
    /// Rendered image width in pixels.
    image_width: u32,
    /// Rendered image height in pixels (derived from width and aspect ratio).
    image_height: u32,
    /// World-space position of the camera.
    center: Point3,
    /// World-space center of the pixel at column 0, row 0.
    pixel00_location: Point3,
    /// World-space step from one pixel to the next along a row.
    pixel_delta_u: Vec3,
    /// World-space step from one row to the next.
    pixel_delta_v: Vec3,
    /// Number of jittered rays cast per pixel.
    samples_per_pixel: u32,
    /// `1 / samples_per_pixel`, cached to turn the per-pixel sum into a mean.
    pixel_samples_scale: f64,
}

impl Camera {
    /// Renders `world` to `writer` as a PPM (P3) image.
    ///
    /// Writes a PPM header (`P3 <w> <h> 255`), then, for each pixel,
    /// averages `samples_per_pixel` jittered rays through that pixel and
    /// emits the result via [`write_color`]. Progress is reported to
    /// stderr a scanline at a time so the rendered image on stdout stays
    /// clean.
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
                    pixel_color += Self::ray_color(&ray, world)
                }
                write_color(writer, pixel_color * self.pixel_samples_scale)?;
            }
        }

        writer.flush()?;
        println!("\n Done!");
        Ok(())
    }

    /// Returns the color seen along `r`.
    ///
    /// On a hit, the surface normal (in `[-1, 1]³`) is remapped into
    /// `[0, 1]³` and used directly as RGB — a debug shading useful for
    /// confirming that geometry and normals are oriented correctly. On a
    /// miss, the ray's normalized `y` component drives a vertical
    /// lerp from white at the horizon to a pale blue overhead, producing
    /// a simple sky gradient.
    fn ray_color(r: &Ray, world: &dyn Hittable) -> Color {
        if let Some(rec) = world.hit(r, Interval::new(0.0, f64::INFINITY)) {
            let direction = Vec3::random_on_hemisphere(&rec.normal);
            return 0.5 * Self::ray_color(&Ray::new(rec.point, direction), world);
        }

        let unit_direction = r.direction.unit_vector();
        let a = 0.5 * (unit_direction.y + 1.0);
        (1.0 - a) * Color::new(1.0, 1.0, 1.0) + a * Color::new(0.5, 0.7, 1.0)
    }

    /// Constructs a camera ray through pixel `(i, j)`, jittered by a
    /// random sub-pixel offset for anti-aliasing.
    fn get_ray(&self, i: u32, j: u32) -> Ray {
        let offset = Self::sample_square();
        let pixel_sample = self.pixel00_location
            + (f64::from(i) + offset.x) * self.pixel_delta_u
            + (f64::from(j) + offset.y) * self.pixel_delta_v;
        let ray_origin = self.center;
        let ray_direction = pixel_sample - self.center;
        Ray::new(ray_origin, ray_direction)
    }

    /// Returns a uniformly random offset inside the unit square centered
    /// on the origin: `x, y ∈ [-0.5, 0.5)`, with `z = 0`.
    fn sample_square() -> Point3 {
        Point3::new(
            rand::random::<f64>() - 0.5,
            rand::random::<f64>() - 0.5,
            0.0,
        )
    }
}
