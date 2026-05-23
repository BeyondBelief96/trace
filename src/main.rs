#![warn(clippy::pedantic)]

mod camera;
mod color;
mod hittable;
mod interval;
mod ray;
mod sphere;
mod vec3;
mod world;

use crate::camera::CameraBuilder;
use crate::sphere::Sphere;
use crate::vec3::Point3;
use crate::world::World;
use std::fs::File;
use std::io::BufWriter;

fn main() -> std::io::Result<()> {
    // World setup

    let mut world = World::new();
    world.add(Sphere::new(Point3::new(0.0, 0.0, -1.0), 0.5));
    world.add(Sphere::new(Point3::new(0.0, -100.5, -1.0), 100.0));

    // Camera setup
    let camera = CameraBuilder::new()
        .aspect_ratio(16.0 / 9.0)
        .image_width(400)
        .build();

    // Render the image
    let file = File::create("image.ppm")?;
    let mut writer = BufWriter::new(file);
    camera.render(&world, &mut writer)?;
    Ok(())
}
