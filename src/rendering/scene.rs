use crate::math::colliders::Collider;
use crate::math::ray::*;

pub struct Scene {
    pub renderables: Vec<Collider>,
}

impl Scene {
    pub fn new() -> Self {
        Scene {
            renderables: Vec::new(),
        }
    }

    pub fn put(&mut self, collider: Collider) {
        self.renderables.push(collider);
    }

    pub fn cast(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<RayHit> {
        let mut best_hit: Option<RayHit> = None;
        let mut earliest_time = t_max;
        for collider in &self.renderables {
            if let Some(hit) = collider.hit(&ray, t_min, earliest_time) {
                earliest_time = hit.hit_fraction;
                best_hit = Some(hit);
            }
        }
        best_hit
    }
}
