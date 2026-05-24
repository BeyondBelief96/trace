//! Ray-intersection abstraction.
//!
//! Defines the [`Hittable`] trait, implemented by anything a ray can hit
//! (spheres, collections of objects, …), and [`HitRecord`], the bundle of
//! information returned at the closest valid intersection.

use std::{fmt::Debug, sync::Arc};

use crate::{
    interval::Interval,
    material::Material,
    ray::Ray,
    vec3::{Point3, Vec3},
};

/// Information about a single ray–surface intersection.
///
/// The `normal` is always stored pointing *against* the incoming ray, with
/// `front_face` recording whether that direction matched the surface's
/// outward normal. This lets shading code treat both sides of a surface
/// uniformly while still knowing which side the ray came from.
#[derive(Clone)]
pub struct HitRecord {
    /// Point of intersection in world space.
    pub point: Point3,
    /// Surface normal at `point`, oriented against the ray's direction.
    pub normal: Vec3,
    pub material: Arc<dyn Material>,
    /// Ray parameter at which the hit occurred (`ray.at(t) == point`).
    pub t: f64,
    /// `true` if the ray hit the outward-facing side of the surface.
    pub front_face: bool,
}

impl HitRecord {
    /// Builds a hit record, orienting `normal` so it always faces the
    /// incoming ray.
    ///
    /// `outward_normal` must be the geometry's true outward normal at
    /// `point` and is assumed to be unit length. The constructor decides
    /// whether the ray struck the front face by checking the sign of
    /// `direction · outward_normal`; the stored normal is flipped on
    /// back-face hits so consumers can dot against it directly.
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

/// Anything a ray can intersect.
///
/// Implementors compute the nearest intersection of `r` whose `t` falls
/// within `t_interval`, returning `None` when no such hit exists.
/// Restricting `t` to an interval lets callers cull hits that are too
/// close (avoiding self-intersection) or farther than the current best
/// candidate (early-out for collections).
pub trait Hittable: Debug {
    fn hit(&self, r: &Ray, t_interval: Interval) -> Option<HitRecord>;
}
