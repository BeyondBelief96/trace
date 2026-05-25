//! Lambertian (matte) diffuse material.
//!
//! Scatters in a cosine-weighted distribution around the surface normal,
//! modeling an ideally diffuse surface. Implemented by adding a random
//! unit vector to the normal — the result biases toward the normal and
//! yields the cosine distribution for free.

use crate::{
    color::Color,
    material::{Material, ScatterResult},
    ray::Ray,
    vec3::Vec3,
};

/// Matte/Diffuse surface defined by its albedo (per-channel reflectance in `[0, 1]`).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Lambertian {
    pub albedo: Color,
}

impl Lambertian {
    pub fn new(albedo: Color) -> Self {
        Self { albedo }
    }
}

impl Material for Lambertian {
    fn scatter(
        &self,
        _: &crate::ray::Ray,
        hit_record: &crate::hittable::HitRecord,
    ) -> Option<crate::material::ScatterResult> {
        let mut scatter_direction = hit_record.normal + Vec3::random_unit_vector();

        // Fall back to the normal when the random vector almost cancels
        // it out — otherwise normalizing the tiny sum would amplify noise.
        if scatter_direction.near_zero() {
            scatter_direction = hit_record.normal
        }

        let scattered = Ray::new(hit_record.point, scatter_direction);
        let attenuation = self.albedo;
        Some(ScatterResult {
            attenuation,
            ray: scattered,
        })
    }
}
