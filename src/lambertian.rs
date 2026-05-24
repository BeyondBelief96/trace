use crate::{
    color::Color,
    material::{Material, ScatterResult},
    ray::Ray,
    vec3::Vec3,
};

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

        // Catch degenerate scatter direction
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
