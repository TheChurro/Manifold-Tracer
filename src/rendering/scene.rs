use crate::math::colliders::Collider;
use crate::math::ray::*;

use crate::rendering::bvh::BoundingVolumeHierarchy;
use crate::rendering::materials::Material;

pub struct Scene {
    pub renderables: Vec<Renderable>,
    pub hierarchy: Option<BoundingVolumeHierarchy>
}

pub struct Renderable {
    pub collider: Collider,
    pub material: Material,
}

impl Scene {
    pub fn new() -> Self {
        Scene {
            renderables: Vec::new(),
            hierarchy: None
        }
    }

    pub fn put(&mut self, collider: Collider, material: Material) {
        self.renderables.push(Renderable {
            collider: collider,
            material: material,
        });
        self.hierarchy = None;
    }

    pub fn compute_hierarchy(&mut self, t_min: f32, t_max: f32) {
        self.hierarchy = Some(BoundingVolumeHierarchy::construct(
            &self.renderables,
            t_min,
            t_max
        ));
    }

    pub fn print_hierarchy(&self) {
        if let &Some(ref hierarchy) = &self.hierarchy {
            hierarchy.print();
        } else {
            println!("None!");
        }
    }

    pub fn cast(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<(RayHit, Material)> {
        if let &Some(ref hierarchy) = &self.hierarchy {
            hierarchy.cast_ray(&self.renderables, ray, t_min, t_max)
        } else {
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
}
