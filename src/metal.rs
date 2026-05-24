use crate::{
    color::Color,
    hittable::HitRecord,
    material::{Material, ScatterResult},
    ray::Ray,
};

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Metal {
    pub albedo: Color,
}

impl Metal {
    pub fn new(albedo: Color) -> Self {
        Self { albedo }
    }
}

impl Material for Metal {
    fn scatter(&self, ray: &Ray, hit_record: &HitRecord) -> Option<ScatterResult> {
        let reflected = ray.direction.reflect(&hit_record.normal);
        let scattered = Ray::new(hit_record.point, reflected);
        let attenuation = self.albedo;
        Some(ScatterResult {
            attenuation,
            ray: scattered,
        })
    }
}
