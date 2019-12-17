use crate::math::quaternion::Quaternion;
use crate::math::vectors::Vec3;
use crate::math::ray::Ray;

use rand::distributions::Uniform;
use rand::{Rng, thread_rng};

pub struct Camera {
    pub location: Vec3,
    pub extents: Vec3,
    pub horizontal: Vec3,
    pub vertical: Vec3,
    pub forward: Vec3,
    pub lens_radius: f32,
    pub orientation: Quaternion,
}

fn random_point_on_disk() -> Vec3 {
    let mut rng = thread_rng();
    let between = Uniform::new(-1.0f32, 1.0f32);
    loop {
        let point = Vec3::new(rng.sample(between), rng.sample(between), 0.0);
        if point.length_sq() < 1.0 {
            return point;
        }
    }
}

impl Camera {
    pub fn new(location: Vec3, target: Vec3, up: Vec3, vertical_fov: f32, aspect: f32, aperture: f32, focus_dist: f32) -> Camera {
        let theta = vertical_fov * std::f32::consts::PI / 180.0;
        let half_height = (theta / 2.0).tan();
        let mut forward = target - location;
        let forward_length = forward.length();
        if forward_length < 0.0001 {
            forward = Vec3::new(0.0, 0.0, -1.0)
        } else {
            forward /= forward_length;
        }
        let rotation = Quaternion::look_at(forward, up);
        let extents = Vec3::new(aspect * half_height, half_height, 1.0);
        Camera {
            location: location,
            extents: extents * focus_dist,
            horizontal: 2.0 * rotation * Vec3::new(aspect * half_height, 0.0, 0.0) * focus_dist,
            vertical: 2.0 * rotation * Vec3::new(0.0, half_height, 0.0) * focus_dist,
            forward: rotation * Vec3::new(0.0, 0.0, 1.0) * focus_dist,
            lens_radius: aperture / 2.0,
            orientation: rotation,
        }
    }

    pub fn bottom_left(&self) -> Vec3 {
        self.location - 0.5f32 * self.horizontal - 0.5f32 * self.vertical + self.forward
    }

    /// Convert a point in camera space into world space.
    pub fn world_ray(&self, u: f32, v: f32) -> Ray {
        let lens_point = self.lens_radius * random_point_on_disk();
        let start_offset = self.location + self.orientation * lens_point;
        Ray::look_at(
            self.location + start_offset,
            self.location + self.forward + (u - 0.5f32) * self.horizontal + (0.5f32 - v) * self.vertical
        )
        // We want to start at top left corner. So the vertical needs to flip how it computes

    }
}
