use std::fmt::Debug;

use crate::{
    interval::Interval,
    ray::Ray,
    vec3::{Point3, Vec3},
};

#[derive(Debug, Copy, Clone, PartialEq, Default)]
pub struct HitRecord {
    pub point: Point3,
    pub normal: Vec3,
    pub t: f64,
    pub front_face: bool,
}

impl HitRecord {
    pub fn new(point: Point3, t: f64, r: &Ray, outward_normal: Vec3) -> Self {
        let front_face = r.direction.dot(outward_normal) < 0.0;
        Self {
            point,
            normal: if front_face {
                outward_normal
            } else {
                -outward_normal
            },
            t,
            front_face,
        }
    }
}

pub trait Hittable: Debug {
    fn hit(&self, r: &Ray, t_interval: Interval) -> Option<HitRecord>;
}
