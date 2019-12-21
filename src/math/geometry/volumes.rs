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
        let mut rng = thread_rng();
        let between = Uniform::new(0.0f32, 1.0);
        const ENABLE_DEBUGGING: bool = false;
        let debug_output = ENABLE_DEBUGGING && rng.sample(between) < 0.00001;
        //self.boundary.hit(ray, t_min, std::f32::MAX)
        // Check for a hit where we enter into our solid
        // Need to check behind as well...
        if let Some(enter_hit) = self.boundary.hit(ray, std::f32::MIN, std::f32::MAX) {
            if let Some(exit_hit) =
                self.boundary
                    .hit(ray, enter_hit.hit_fraction + 0.0001, std::f32::MAX)
            {
                // Clamp to start time of the ray (this is mostly for
                // recursive constant volumes. I don't know what that
                // would look like)
                let t_enter = enter_hit.hit_fraction.max(t_min);
                let t_exit = exit_hit.hit_fraction.min(t_max);
                if debug_output {
                    println!("Point in volume!");
                    println!("Enter/Exit = {}/{}", enter_hit.hit_fraction, exit_hit.hit_fraction);
                    println!("t_enter/t_exit = {}/{}", t_enter, t_exit);
                }
                if t_enter >= t_exit {
                    return None;
                }
                // Clamp to initial position of ray
                let t_enter = t_enter.max(0.0);
                let direction_length = ray.direction.length();
                let distance_in_boundary = (t_exit - t_enter) * direction_length;

                let hit_distance = -(1.0 / self.density) * rng.sample(between).ln();
                if hit_distance <= distance_in_boundary {
                    let hit_time = t_enter + hit_distance / direction_length;
                    let ray_hit = RayHit{
                        hit_fraction: hit_time,
                        location: ray.point_at_parameter(hit_time),
                        normal: UnitSphere.sample(&mut rng).into(),
                        u: 0.5 * (enter_hit.u + exit_hit.u),
                        v: 0.5 * (enter_hit.v + exit_hit.v)
                    };
                    if debug_output {
                        println!("Hit Distance: {} / {}", hit_distance, distance_in_boundary);
                        println!("Hit: {}\n", &ray_hit);
                    }
                    return Some(ray_hit);
                    // return Some(ray_hit);
                } else if debug_output {
                    println!("Hit Distance: {} / {}", hit_distance, distance_in_boundary);
                    println!("NO HIT==============\n");
                }
            }
        }
        None
    }
}
