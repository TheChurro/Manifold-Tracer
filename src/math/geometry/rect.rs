use crate::math::geometry::aabb::AABBGeometry;
use crate::math::ray::{Ray, RayCollidable, RayHit};
use crate::math::vectors::Vec3;

#[derive(Clone, Copy, Debug)]
pub struct RectGeometry {
    pub center: Vec3,
    pub width: f32,
    pub height: f32,
}

impl RectGeometry {
    pub fn new(center: Vec3, width: f32, height: f32) -> RectGeometry {
        RectGeometry {
            center: center,
            width: width,
            height: height,
        }
    }
}

impl RayCollidable for RectGeometry {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<RayHit> {
        let t = if ray.direction.x.abs() < 0.0001 {
            let start = ray.point_at_parameter(t_min);
            if (start.z - self.center.z).abs() < 0.0001 {
                t_min
            } else {
                return None;
            }
        } else {
            (self.center.z - ray.origin.z) / ray.direction.z
        };
        if t < t_min || t > t_max {
            return None;
        }
        let target_pos = ray.point_at_parameter(t);
        let off = target_pos - self.center;
        let off_test = off.abs();
        if off_test.x >= self.width / 2.0 || off_test.y >= self.height / 2.0 {
            return None;
        }
        Some(RayHit {
            hit_fraction: t,
            location: target_pos,
            normal: -Vec3::forward(),
            u: off.x / self.width + 0.5,
            v: off.y / self.height + 0.5,
        })
    }

    fn bounding_box(&self, _t_min: f32, _t_max: f32) -> Option<AABBGeometry> {
        Some(AABBGeometry {
            center: self.center,
            extents: Vec3::new(self.width / 2.0, self.height / 2.0, 0.0001),
        })
    }
}
