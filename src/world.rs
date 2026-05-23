use crate::{
    hittable::{HitRecord, Hittable},
    interval::Interval,
};

#[derive(Default, Debug)]
pub struct World {
    pub objects: Vec<Box<dyn Hittable>>,
}

impl World {
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
        }
    }

    pub fn clear(&mut self) {
        self.objects.clear();
    }

    pub fn add<H: Hittable + 'static>(&mut self, object: H) {
        self.objects.push(Box::new(object));
    }
}

impl Hittable for World {
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
