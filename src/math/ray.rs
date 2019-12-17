use crate::math::vectors::Vec3;

pub struct Ray {
    pub origin: Vec3,
    pub direction: Vec3,
}

impl Ray {
    pub fn look_at(origin: Vec3, target: Vec3) -> Ray {
        Ray {
            origin: origin,
            direction: (target - origin).normalized(),
        }
    }

    pub fn point_at_parameter(self, time: f32) -> Vec3 {
        self.origin + time * self.direction
    }
}
