use crate::math::colliders::Collider;
use crate::math::ray::*;

pub struct Scene {
    pub renderables: Vec<Collider>,
}

impl Scene {
    pub fn new() -> Self {
        Scene {
            renderables: Vec::new()
        }
    }

    pub fn put(&mut self, collider: Collider) {
        self.renderables.push(collider);
    }

    pub fn cast(&self, ray: &Ray) -> Option<RayHit> {
        let mut best_hit: Option<RayHit> = None;
        for collider in &self.renderables {
            if let Some(hit) = collider.hit(&ray) {
                best_hit = match best_hit {
                    Some(old_hit) => {
                        if old_hit.hit_fraction < hit.hit_fraction {
                            Some(old_hit)
                        } else {
                            Some(hit)
                        }
                    }
                    None => Some(hit),
                };
            }
        }
        best_hit
    }
}
