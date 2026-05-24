//! Binary entry point.
//!
//! Builds a small scene, configures a camera, and renders the result to
//! `image.ppm` in the current working directory. All module wiring lives
//! here; the rendering itself is delegated to [`camera::Camera::render`].

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
    let mut world = World::new();
    world.add(Sphere::new(Point3::new(0.0, 0.0, -1.0), 0.5));
    world.add(Sphere::new(Point3::new(0.0, -100.5, -1.0), 100.0));

    let camera = CameraBuilder::new()
        .aspect_ratio(16.0 / 9.0)
        .image_width(400)
        .samples_per_pixel(100)
        .maximum_depth(50)
        .build();

    let file = File::create("image.ppm")?;
    let mut writer = BufWriter::new(file);
    camera.render(&world, &mut writer)?;
    Ok(())
}
