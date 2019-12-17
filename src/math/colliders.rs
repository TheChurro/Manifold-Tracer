use crate::math::ray::{Ray, RayCollidable, RayHit};
use crate::math::vectors::Vec3;

pub enum Collider {
    Sphere(SphereGeometry),
}

impl RayCollidable for Collider {
    fn hit(&self, ray: &Ray) -> Option<RayHit> {
        match self {
            &Self::Sphere(ref geometry) => geometry.hit(ray),
        }
    }
}

impl Collider {
    pub fn new(geometry: SphereGeometry) -> Collider {
        Collider::Sphere(geometry)
    }
}

pub struct SphereGeometry {
    pub center: Vec3,
    pub radius: f32,
}

fn smaller_non_zero(a: f32, b: f32) -> Option<f32> {
    match (a < 0f32, b < 0f32) {
        (true, true) => None,
        (true, false) => Some(b),
        (false, true) => Some(a),
        (false, false) => Some(f32::min(a, b)),
    }
}

impl RayCollidable for SphereGeometry {
    fn hit(&self, ray: &Ray) -> Option<RayHit> {
        let offset = ray.origin - self.center;
        let a = ray.direction.length_sq();
        let b = 2.0f32 * offset.dot(&ray.direction);
        let c = offset.length_sq() - self.radius * self.radius;
        let descriminant = b * b - 4f32 * a * c;
        if descriminant > 0f32 {
            let sqrt_descriminant = f32::sqrt(descriminant);
            // Compute the hit time, however, ensure that hit occurs
            // after the ray starts moving.
            let hit_time = {
                let hit_a = (-b + sqrt_descriminant) / (a + a);
                let hit_b = (-b - sqrt_descriminant) / (a + a);
                smaller_non_zero(hit_a, hit_b)
            };
            if let Some(time) = hit_time {
                let location = ray.point_at_parameter(time);
                Some(RayHit {
                    hit_fraction: time,
                    location: location,
                    normal: (location - self.center).normalized(),
                })
            } else {
                None
            }
        } else {
            None
        }
    }
}
