use crate::math::geometry::aabb::AABBGeometry;
use crate::math::vectors::Vec3;

#[derive(Debug)]
pub struct Ray {
    pub origin: Vec3,
    pub direction: Vec3,
    pub cast_time: f32,
}

impl Ray {
    pub fn new(origin: Vec3, direction: Vec3) -> Ray {
        Ray {
            origin: origin,
            direction: direction.normalized(),
            cast_time: 0.0,
        }
    }

    pub fn look_at(origin: Vec3, target: Vec3) -> Ray {
        Ray {
            origin: origin,
            direction: (target - origin).normalized(),
            cast_time: 0.0,
        }
    }

    pub fn cast_at(self, time: f32) -> Ray {
        Ray {
            cast_time: time,
            ..self
        }
    }

    pub fn point_at_parameter(&self, time: f32) -> Vec3 {
        self.origin + time * self.direction
    }
}

#[derive(Debug)]
pub struct RayHit {
    pub hit_fraction: f32,
    pub location: Vec3,
    pub normal: Vec3,
    pub u: f32,
    pub v: f32,
}

use std::fmt;
impl fmt::Display for RayHit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "RayHit {{ location: {} | normal: {} | uv: ({}, {}) }} at {}",
            self.location, self.normal, self.u, self.v, self.hit_fraction
        )
    }
}

pub trait RayCollidable {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<RayHit>;
    fn bounding_box(&self, t_min: f32, t_max: f32) -> Option<AABBGeometry>;
}
