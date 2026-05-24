//! Surface scattering models.
//!
//! A [`Material`] decides what happens when a ray hits its surface: which
//! direction (if any) the ray scatters in, and the color attenuation
//! applied along the way. Returning `None` from [`Material::scatter`]
//! absorbs the ray entirely, contributing black to the final pixel.
//!
//! Concrete implementations live in submodules — see [`lambertian`] and
//! [`metal`].

pub mod dielectric;
pub mod lambertian;
pub mod metal;

use std::fmt::Debug;

use crate::{color::Color, hittable::HitRecord, ray::Ray};

/// Outcome of a successful scatter: the new ray and its color attenuation.
#[derive(Debug, Clone)]
pub struct ScatterResult {
    pub attenuation: Color,
    pub ray: Ray,
}

/// A surface scattering model. See module docs.
pub trait Material: Debug + Send + Sync {
    fn scatter(&self, ray: &Ray, hit_record: &HitRecord) -> Option<ScatterResult>;
}
