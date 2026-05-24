use std::fmt::Debug;

use crate::{color::Color, hittable::HitRecord, ray::Ray};

/// Represents the result of scattering a ray off a surface.
///
/// Contains the attenuation color and the scattered ray.
#[derive(Debug, Clone)]
pub struct ScatterResult {
    pub attenuation: Color,
    pub ray: Ray,
}

pub trait Material: Debug + Send + Sync {
    fn scatter(&self, ray: &Ray, hit_record: &HitRecord) -> Option<ScatterResult>;
}
