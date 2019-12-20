use crate::math::geometry::aabb::AABBGeometry;
use crate::math::ray::{Ray, RayCollidable, RayHit};

use rand::{Rng, thread_rng};
use rand::distributions::Uniform;
use rand_distr::{UnitSphere, Distribution};

pub struct ConstantVolume {
    pub boundary: Box<dyn RayCollidable>,
    pub density: f32
}

impl RayCollidable for ConstantVolume {
    fn bounding_box(&self, t_min: f32, t_max: f32) -> Option<AABBGeometry> {
        self.boundary.bounding_box(t_min, t_max)
    }

    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<RayHit> {
        // Check for a hit where we enter into our solid
        if let Some(enter_hit) = self.boundary.hit(ray, std::f32::MIN, std::f32::MAX) {
            if let Some(exit_hit) =
                self.boundary
                    .hit(ray, enter_hit.hit_fraction + 0.00001, std::f32::MAX)
            {
                // Clamp to start time of the ray (this is mostly for
                // recursive constant volumes. I don't know what that
                // would look like)
                let t_enter = enter_hit.hit_fraction.max(t_min);
                let t_exit = exit_hit.hit_fraction.min(t_max);
                if t_enter >= t_exit {
                    return None;
                }
                // Clamp to initial position of ray
                let t_enter = t_enter.max(0.0);
                let direction_length = ray.direction.length();
                let distance_in_boundary = (t_exit - t_enter) * direction_length;
                let mut rng = thread_rng();
                let hit_distance = -(1.0 / self.density) * rng.sample(Uniform::new(0.0f32, 1.0)).ln();
                if hit_distance < distance_in_boundary {
                    let hit_time = t_enter + hit_distance / direction_length;
                    return Some(RayHit{
                        hit_fraction: hit_time,
                        location: ray.point_at_parameter(hit_time),
                        normal: UnitSphere.sample(&mut rng).into(),
                        u: 0.5 * (enter_hit.u + exit_hit.u),
                        v: 0.5 * (enter_hit.v + exit_hit.v)
                    });
                }
            }
        }
        None
    }
}
