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
mod material;
mod ray;
mod vec3;
mod world;

use crate::camera::CameraBuilder;
use crate::color::Color;
use crate::hittable::sphere::Sphere;
use crate::material::dielectric::Dielectric;
use crate::material::lambertian::Lambertian;
use crate::material::metal::Metal;
use crate::vec3::Point3;
use crate::world::World;
use std::fs::File;
use std::io::BufWriter;
use std::sync::Arc;

fn main() -> std::io::Result<()> {
    let mut world = World::new();
    let material_ground = Arc::new(Lambertian::new(Color::new(0.8, 0.8, 0.0)));
    let material_center = Arc::new(Lambertian::new(Color::new(0.1, 0.2, 0.5)));
    let material_left = Arc::new(Dielectric::new(1.5));
    let material_bubble = Arc::new(Dielectric::new(1.0 / 1.5));
    let material_right = Arc::new(Metal::new(Color::new(0.8, 0.6, 0.2), 0.1));
    world.add(Sphere::new(
        Point3::new(0.0, -100.5, -1.0),
        100.0,
        material_ground,
    ));
    world.add(Sphere::new(
        Point3::new(0.0, 0.0, -1.2),
        0.5,
        material_center,
    ));

    world.add(Sphere::new(
        Point3::new(-1.0, 0.0, -1.0),
        0.5,
        material_left,
    ));
    world.add(Sphere::new(
        Point3::new(-1.0, 0.0, -1.0),
        0.4,
        material_bubble,
    ));
    world.add(Sphere::new(
        Point3::new(1.0, 0.0, -1.0),
        0.5,
        material_right,
    ));

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
