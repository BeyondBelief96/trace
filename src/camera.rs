use std::io::{self, Write};

use crate::{
    color::{Color, write_color},
    hittable::Hittable,
    interval::Interval,
    ray::Ray,
    vec3::{Point3, Vec3},
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CameraBuilder {
    pub aspect_ratio: f64,
    pub image_width: u32,
    pub samples_per_pixel: u32,
}

impl CameraBuilder {
    pub fn new() -> Self {
        Self {
            aspect_ratio: 16.0 / 9.0,
            image_width: 400,
            samples_per_pixel: 10,
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

    pub fn build(self) -> Camera {
        #[allow(clippy::cast_possible_truncation)]
        #[allow(clippy::cast_sign_loss)]
        // Calculate image height based on aspect ratio and image width, ensuring it's at least 1
        let image_height = (f64::from(self.image_width) / self.aspect_ratio) as u32;
        let image_height = if image_height < 1 { 1 } else { image_height };

        // Viewport
        let focal_length = 1.0;
        let viewport_height = 2.0;
        let viewport_width =
            viewport_height * (f64::from(self.image_width) / f64::from(image_height));
        let camera_center = Point3::new(0.0, 0.0, 0.0);

        let viewport_u = Vec3::new(viewport_width, 0.0, 0.0);
        let viewport_v = Vec3::new(0.0, -viewport_height, 0.0);

        // Calculate the horizontal and vertical delta vectors from pixel to pixel
        let pixel_delta_u = viewport_u / f64::from(self.image_width);
        let pixel_delta_v = viewport_v / f64::from(image_height);

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

/// Defines a camera that can be used to render an image.
pub struct Camera {
    /// Rendered image width
    image_width: u32,
    /// Rendered image height
    image_height: u32,
    /// Camera center
    center: Point3,
    /// Location of the pixel at (0, 0) in the image.
    pixel00_location: Point3,
    /// Vector from pixel to pixel in the u direction
    pixel_delta_u: Vec3,
    /// Vector from pixel to pixel in the v direction
    pixel_delta_v: Vec3,
    /// Count of random samples per pixel
    samples_per_pixel: u32,
    /// Color scale factor for a sum of pixel samples
    pixel_samples_scale: f64,
}

impl Camera {
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

    fn ray_color(r: &Ray, world: &dyn Hittable) -> Color {
        if let Some(rec) = world.hit(r, Interval::new(0.0, f64::INFINITY)) {
            return 0.5 * Color::new(rec.normal.x + 1.0, rec.normal.y + 1.0, rec.normal.z + 1.0);
        }

        // If we miss, return the background color.
        let unit_direction = r.direction.unit_vector();
        let a = 0.5 * (unit_direction.y + 1.0);
        (1.0 - a) * Color::new(1.0, 1.0, 1.0) + a * Color::new(0.5, 0.7, 1.0)
    }

    /// Constructs a camera ray originating from the camera center and passing through a randomly sampled
    /// point around the pixel (i, j).
    fn get_ray(&self, i: u32, j: u32) -> Ray {
        let offset = Self::sample_square();
        let pixel_sample = self.pixel00_location
            + (f64::from(i) + offset.x) * self.pixel_delta_u
            + (f64::from(j) + offset.y) * self.pixel_delta_v;
        let ray_origin = self.center;
        let ray_direction = pixel_sample - self.center;
        Ray::new(ray_origin, ray_direction)
    }

    /// Returns a random point in the square [-0.5, -0.5] to [0.5, 0.5] assuming an origin of [0, 0]
    fn sample_square() -> Point3 {
        Point3::new(
            rand::random::<f64>() - 0.5,
            rand::random::<f64>() - 0.5,
            0.0,
        )
    }
}
