//! Sphere primitive and its ray-intersection routine.

use std::sync::Arc;

use crate::{
    hittable::{HitRecord, Hittable},
    interval::Interval,
    material::Material,
    vec3::Vec3,
};

/// A sphere defined by a center point and a radius.
///
/// Negative radii are accepted and produce "inside-out" spheres — useful
/// for hollow shapes where the outward normal should point inward.
#[derive(Clone, Debug)]
pub struct Sphere {
    pub center: Vec3,
    pub radius: f64,
    pub material: Arc<dyn Material>,
}

impl Sphere {
    /// Constructs a sphere from its center and radius.
    pub const fn new(center: Vec3, radius: f64, material: Arc<dyn Material>) -> Self {
        Self {
            center,
            radius,
            material,
        }
    }
}

impl Hittable for Sphere {
    /// Intersects a ray with this sphere and returns the nearest hit
    /// whose `t` parameter lies strictly inside `t_interval`.
    ///
    /// Derivation: a point `P` is on the sphere when
    /// `(P - center) · (P - center) = radius²`. Substituting the ray
    /// `P = origin + t·d` and letting `oc = center - origin` yields a
    /// quadratic in `t`:
    ///
    /// ```text
    /// (d·d) t² - 2 (d·oc) t + (oc·oc - r²) = 0
    /// ```
    ///
    /// Setting `h = d·oc` lets us drop the factor of 2 from the standard
    /// quadratic formula:
    ///
    /// ```text
    /// t = (h ± sqrt(h² - a·c)) / a       where a = d·d, c = oc·oc - r²
    /// ```
    ///
    /// A negative discriminant means the ray misses the sphere entirely.
    /// Otherwise the smaller root is tried first (the nearer intersection);
    /// if it falls outside `t_interval`, the farther root is tried before
    /// giving up.
    fn hit(&self, r: &crate::ray::Ray, t_interval: Interval) -> Option<crate::hittable::HitRecord> {
        let oc = self.center - r.origin;
        let a = r.direction.length_squared();
        let h = r.direction.dot(oc);
        let c = oc.length_squared() - self.radius * self.radius;
        let discriminant = h * h - a * c;
        if discriminant < 0.0 {
            return None;
        }
        let sqrt_discriminant = discriminant.sqrt();
        let mut root = (h - sqrt_discriminant) / a;

        if !t_interval.surrounds(root) {
            root = (h + sqrt_discriminant) / a;
            if !t_interval.surrounds(root) {
                return None;
            }
        }

        let point = r.at(root);
        let outward_normal = (point - self.center) / self.radius;
        Some(HitRecord::new(
            point,
            root,
            r,
            outward_normal,
            Arc::clone(&self.material),
        ))
    }
}
