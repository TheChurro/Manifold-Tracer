use crate::math::colliders::Collider;
use crate::math::ray::*;

use crate::rendering::materials::Material;

pub struct Scene {
    pub renderables: Vec<Renderable>,
}

pub struct Renderable {
    pub collider: Collider,
    pub material: Material,
}

impl Scene {
    pub fn new() -> Self {
        Scene {
            renderables: Vec::new(),
        }
    }

    pub fn put(&mut self, collider: Collider, material: Material) {
        self.renderables.push(Renderable {
            collider: collider,
            material: material,
        });
    }

    pub fn cast(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<(RayHit, Material)> {
        let mut best_hit: Option<(RayHit, Material)> = None;
        let mut earliest_time = t_max;
        for renderable in &self.renderables {
            if let Some(hit) = renderable.collider.hit(&ray, t_min, earliest_time) {
                earliest_time = hit.hit_fraction;
                best_hit = Some((hit, renderable.material));
            }
        }
        best_hit
    }
}
