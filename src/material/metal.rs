//! Metallic material with optional fuzziness.
//!
//! Scatters by mirror-reflecting about the surface normal, then perturbs
//! the reflected direction by a random offset scaled by `fuzz` — `0` is a
//! perfect mirror, `1` is a maximally blurry brushed surface. `albedo`
//! tints the reflected light per channel.

use crate::{
    color::Color,
    hittable::HitRecord,
    material::{Material, ScatterResult},
    ray::Ray,
    vec3::Vec3,
};

/// Reflective surface tinted by `albedo`, blurred by `fuzz` ∈ `[0, 1]`.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Metal {
    pub albedo: Color,
    pub fuzz: f64,
}

impl Metal {
    /// `fuzz` is clamped to `[0, 1]`.
    pub fn new(albedo: Color, fuzz: f64) -> Self {
        let fuzz = fuzz.clamp(0.0, 1.0);
        Self { albedo, fuzz }
    }
}

impl Material for Metal {
    fn scatter(&self, ray: &Ray, hit_record: &HitRecord) -> Option<ScatterResult> {
        let reflected = ray.direction.reflect(&hit_record.normal).unit_vector()
            + (self.fuzz * Vec3::random_unit_vector());
        let scattered = Ray::new(hit_record.point, reflected);
        let attenuation = self.albedo;
        Some(ScatterResult {
            attenuation,
            ray: scattered,
        })
    }
}
