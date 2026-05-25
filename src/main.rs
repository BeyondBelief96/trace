//! Binary entry point.
//!
//! Builds the "final scene" from *Ray Tracing in One Weekend* — a ground
//! plane, three feature spheres, and a field of randomly placed small
//! spheres with random materials — then renders it to `image.ppm`.

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
use crate::vec3::{Point3, Vec3};
use crate::world::World;
use std::fs::File;
use std::io::BufWriter;
use std::sync::Arc;

fn main() -> std::io::Result<()> {
    let world = build_scene();

    let camera = CameraBuilder::new()
        .aspect_ratio(16.0 / 9.0)
        .image_width(1200)
        .samples_per_pixel(500)
        .maximum_depth(50)
        .vertical_fov_deg(20.0)
        .look_from(Point3::new(13.0, 2.0, 3.0))
        .look_at(Point3::new(0.0, 0.0, 0.0))
        .up(Vec3::new(0.0, 1.0, 0.0))
        .defocus_angle_deg(0.6)
        .focus_distance(10.0)
        .build();

    let file = File::create("image.ppm")?;
    let mut writer = BufWriter::new(file);
    camera.render(&world, &mut writer)?;
    Ok(())
}

fn build_scene() -> World {
    let mut world = World::new();

    // Ground: a huge sphere acting as a flat plane.
    let ground_material = Arc::new(Lambertian::new(Color::new(0.5, 0.5, 0.5)));
    world.add(Sphere::new(
        Point3::new(0.0, -1000.0, 0.0),
        1000.0,
        ground_material,
    ));

    // Field of small spheres on a 22x22 grid, jittered within each cell.
    // Material is randomized: mostly diffuse, some metal, a few glass.
    // The keep-out check skips any sphere that would overlap the large
    // dielectric at (4, 0.2, 0).
    let keep_out_center = Point3::new(4.0, 0.2, 0.0);
    for a in -11..11 {
        for b in -11..11 {
            let center = Point3::new(
                f64::from(a) + 0.9 * rand::random::<f64>(),
                0.2,
                f64::from(b) + 0.9 * rand::random::<f64>(),
            );

            if (center - keep_out_center).length() <= 0.9 {
                continue;
            }

            let choose_mat = rand::random::<f64>();
            if choose_mat < 0.8 {
                let albedo = Color::random() * Color::random();
                let sphere_material = Arc::new(Lambertian::new(albedo));
                world.add(Sphere::new(center, 0.2, sphere_material));
            } else if choose_mat < 0.95 {
                let albedo = Color::random_in_range(0.5, 1.0);
                let fuzz = rand::random_range(0.0..0.5);
                let sphere_material = Arc::new(Metal::new(albedo, fuzz));
                world.add(Sphere::new(center, 0.2, sphere_material));
            } else {
                let sphere_material = Arc::new(Dielectric::new(1.5));
                world.add(Sphere::new(center, 0.2, sphere_material));
            }
        }
    }

    // Three feature spheres along the z = 0 line.
    let material1 = Arc::new(Dielectric::new(1.5));
    world.add(Sphere::new(Point3::new(0.0, 1.0, 0.0), 1.0, material1));

    let material2 = Arc::new(Lambertian::new(Color::new(0.4, 0.2, 0.1)));
    world.add(Sphere::new(Point3::new(-4.0, 1.0, 0.0), 1.0, material2));

    let material3 = Arc::new(Metal::new(Color::new(0.7, 0.6, 0.5), 0.0));
    world.add(Sphere::new(Point3::new(4.0, 1.0, 0.0), 1.0, material3));

    world
}
