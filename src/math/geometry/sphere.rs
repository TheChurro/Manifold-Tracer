use crate::math::geometry::aabb::AABBGeometry;
use crate::math::ray::{Ray, RayCollidable, RayHit};
use crate::math::vectors::Vec3;

pub struct SphereGeometry {
    pub center: Vec3,
    pub radius: f32,
}

impl SphereGeometry {
    pub fn new(center: Vec3, radius: f32) -> SphereGeometry {
        SphereGeometry {
            center: center,
            radius: radius,
        }
    }
    pub fn offset(&self, offset: Vec3) -> SphereGeometry {
        SphereGeometry {
            center: self.center + offset,
            radius: self.radius,
        }
    }
}

fn smallest_bounded(a: f32, b: f32, min: f32, max: f32) -> Option<f32> {
    match (min <= a && a <= max, min <= b && b <= max) {
        (false, false) => None,
        (false, true) => Some(b),
        (true, false) => Some(a),
        (true, true) => Some(f32::min(a, b)),
    }
}

impl RayCollidable for SphereGeometry {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<RayHit> {
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
                smallest_bounded(hit_a, hit_b, t_min, t_max)
            };
            if let Some(time) = hit_time {
                let location = ray.point_at_parameter(time);
                let rel_point = location - self.center;
                let phi = rel_point.z.atan2(rel_point.x);
                let theta = rel_point.y.asin();
                use std::f32::consts::FRAC_PI_2;
                use std::f32::consts::PI;
                Some(RayHit {
                    hit_fraction: time,
                    location: location,
                    normal: (location - self.center) / self.radius,
                    u: 1.0 - (phi + PI) / (2.0 * PI),
                    v: (theta + FRAC_PI_2) / PI,
                })
            } else {
                None
            }
        } else {
            None
        }
    }

    fn bounding_box(&self, _t_min: f32, _t_max: f32) -> Option<AABBGeometry> {
        let radius = Vec3::all(self.radius.abs());
        Some(AABBGeometry {
            center: self.center,
            extents: radius,
        })
    }
}
