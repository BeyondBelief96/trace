//! A scene composed of any number of [`Hittable`] objects.
//!
//! [`World`] owns a heterogeneous list of trait objects and itself
//! implements [`Hittable`], so it can be passed wherever a single object
//! is expected. Intersecting a ray against the world returns the closest
//! hit across all contained objects.

use crate::{
    hittable::{HitRecord, Hittable},
    interval::Interval,
};

/// A collection of hittable objects forming the renderable scene.
#[derive(Default, Debug)]
pub struct World {
    /// Heap-allocated, dynamically dispatched scene objects.
    pub objects: Vec<Box<dyn Hittable>>,
}

impl World {
    /// Creates an empty world.
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
        }
    }

    /// Removes all objects from the world.
    pub fn clear(&mut self) {
        self.objects.clear();
    }

    /// Adds an object to the world, taking ownership of it.
    ///
    /// `object` is moved onto the heap and stored as a trait object, which
    /// is what allows the world to hold a mix of concrete `Hittable`
    /// implementations (spheres, future primitives, nested worlds…) in a
    /// single list. The `'static` bound forbids stashing references with
    /// shorter lifetimes than the world itself.
    pub fn add<H: Hittable + 'static>(&mut self, object: H) {
        self.objects.push(Box::new(object));
    }
}

impl Hittable for World {
    /// Returns the closest hit across all objects whose `t` lies inside
    /// `t_interval`, or `None` if every object misses.
    ///
    /// The interval's upper bound is tightened to the best `t` seen so far
    /// as the loop progresses, so subsequent objects can early-out on hits
    /// that are farther than the current closest.
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
