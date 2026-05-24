//! A scene composed of any number of [`Hittable`] objects.
//!
//! [`World`] owns a heterogeneous list of trait objects and itself
//! implements [`Hittable`], so it can be passed wherever a single hittable
//! is expected. Intersecting a ray returns the closest hit across all
//! contained objects.

use crate::{
    hittable::{HitRecord, Hittable},
    interval::Interval,
};

/// A collection of hittable objects forming the renderable scene.
#[derive(Default, Debug)]
pub struct World {
    pub objects: Vec<Box<dyn Hittable>>,
}

impl World {
    /// Creates an empty world.
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
        }
    }

    /// Removes all objects.
    pub fn clear(&mut self) {
        self.objects.clear();
    }

    /// Adds an object to the world, boxing it as a trait object so the
    /// list can hold a mix of concrete `Hittable` types.
    pub fn add<H: Hittable + 'static>(&mut self, object: H) {
        self.objects.push(Box::new(object));
    }
}

impl Hittable for World {
    /// Returns the closest hit inside `t_interval`, or `None` if every
    /// object misses. The interval's upper bound is tightened to the best
    /// `t` seen so far as we iterate, letting later objects early-out.
    fn hit(&self, r: &crate::ray::Ray, t_interval: Interval) -> Option<crate::hittable::HitRecord> {
        let mut closest_t_so_far = t_interval.max;
        let mut hit_anything: Option<HitRecord> = None;
        for object in &self.objects {
            if let Some(record) = object.hit(r, Interval::new(t_interval.min, closest_t_so_far)) {
                closest_t_so_far = record.t;
                hit_anything = Some(record);
            }
        }

        hit_anything
    }
}
