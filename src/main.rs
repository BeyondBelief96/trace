use std::fs::File;
use std::io::{BufWriter, Write};

use crate::color::{Color, write_color};
use crate::ray::Ray;
use crate::vec3::{Point3, Vec3};

mod color;
mod ray;
mod vec3;

pub fn hit_sphere(center: Vec3, radius: f64, r: &Ray) -> bool {
    let oc = r.origin - center;
    let a = r.direction.dot(r.direction);
    let b = -2.0 * oc.dot(r.direction);
    let c = oc.dot(oc) - radius * radius;
    let discriminant = b * b - 4.0 * a * c;
    discriminant >= 0.0
}

pub fn ray_color(r: &ray::Ray) -> Color {
    if hit_sphere(Point3::new(0.0, 0.0, -1.0), 0.5, r) {
        return Color::new(1.0, 0.0, 0.0);
    }

    let unit_direction = Vec3::unit_vector(&r.direction);
    let a = 0.5 * (unit_direction.y + 1.0);
    (1.0 - a) * Color::new(1.0, 1.0, 1.0) + a * Color::new(0.5, 0.7, 1.0)
}

fn main() -> std::io::Result<()> {
    let aspect_ratio = 16.0 / 9.0;
    let image_width = 400;

    // Calculate the image height and ensure that it's atleast 1.
    #[allow(clippy::cast_possible_truncation)]
    let image_height = (f64::from(image_width) / aspect_ratio) as i32;
    let image_height = if image_height < 1 { 1 } else { image_height };

    // Camera setup
    let focal_length = 1.0;
    let viewport_height = 2.0;

    let viewport_width = viewport_height * (f64::from(image_width) / f64::from(image_height));
    let camera_center = Vec3::new(0.0, 0.0, 0.0);

    // Calculate the vectors across the horizontal and down the vertical viewport edges.
    let viewport_u = Vec3::new(viewport_width, 0.0, 0.0);
    let viewport_v = Vec3::new(0.0, -viewport_height, 0.0);

    // Calculate the horizontal and vertical delta vectors from pixel to pixel
    let pixel_delta_u = viewport_u / f64::from(image_width);
    let pixel_delta_v = viewport_v / f64::from(image_height);

    // Calculate the location of the upper left pixel
    let viewport_upper_left =
        camera_center - Vec3::new(0.0, 0.0, focal_length) - (viewport_u / 2.0) - (viewport_v / 2.0);
    let pixel00_location = viewport_upper_left + 0.5 * (pixel_delta_u + pixel_delta_v);

    // Render the image

    let file = File::create("image.ppm")?;
    let mut writer = BufWriter::new(file);
    writeln!(writer, "P3")?;
    writeln!(writer, "{image_width} {image_height}")?;
    writeln!(writer, "255")?;

    for j in 0..image_height {
        println!("\rScanline remaining: {}", image_height - j);
        for i in 0..image_width {
            let pixel_center = pixel00_location + (i * pixel_delta_u) + (j * pixel_delta_v);
            let ray_direction = pixel_center - camera_center;
            let ray = Ray::new(camera_center, ray_direction);
            let pixel_color = ray_color(&ray);
            write_color(&mut writer, pixel_color)?;
        }
    }

    writer.flush()?;
    println!("\nDone.");

    Ok(())
}
