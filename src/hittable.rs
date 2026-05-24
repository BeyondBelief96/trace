//! Ray-intersection abstraction.
//!
//! [`Hittable`] is implemented by anything a ray can hit; [`HitRecord`]
//! bundles the information returned at the closest valid intersection.
//! The stored `normal` always points *against* the incoming ray, with
//! `front_face` recording whether that matched the surface's outward
//! normal — so shading code can treat both sides uniformly while still
//! knowing which side was hit.
//!
//! Concrete primitives live in submodules — see [`sphere`].

pub mod sphere;

use std::{fmt::Debug, sync::Arc};

use crate::{
    interval::Interval,
    material::Material,
    ray::Ray,
    vec3::{Point3, Vec3},
};

/// A single ray–surface intersection.
#[derive(Clone)]
pub struct HitRecord {
    pub point: Point3,
    /// Oriented against the incoming ray (see module docs).
    pub normal: Vec3,
    pub material: Arc<dyn Material>,
    /// Ray parameter at the hit: `ray.at(t) == point`.
    pub t: f64,
    /// `true` if the ray hit the outward-facing side.
    pub front_face: bool,
}

impl HitRecord {
    /// Builds a hit record, flipping `outward_normal` if the ray hit a
    /// back face so the stored `normal` always faces the incoming ray.
    /// `outward_normal` is assumed unit length.
    pub fn new(
        point: Point3,
        t: f64,
        r: &Ray,
        outward_normal: Vec3,
        material: Arc<dyn Material>,
    ) -> Self {
        let front_face = r.direction.dot(outward_normal) < 0.0;
        Self {
            point,
            normal: if front_face {
                outward_normal
            } else {
                -outward_normal
            },
            material,
            t,
            front_face,
        }
    }
}

/// Anything a ray can intersect. Implementors return the nearest hit
/// whose `t` lies inside `t_interval`, or `None` for a miss.
pub trait Hittable: Debug {
    fn hit(&self, r: &Ray, t_interval: Interval) -> Option<HitRecord>;
}
