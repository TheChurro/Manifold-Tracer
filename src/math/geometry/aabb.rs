use std::ops::{Add, AddAssign, Mul};

use crate::math::quaternion::Quaternion;
use crate::math::ray::Ray;
use crate::math::vectors::Vec3;

#[derive(Debug, Clone, Copy)]
pub struct AABBGeometry {
    pub center: Vec3,
    pub extents: Vec3,
}

impl AABBGeometry {
    pub fn from_points(a: Vec3, b: Vec3) -> AABBGeometry {
        let center = (a + b) / 2.0;
        let extents = (a - b).abs() / 2.0;
        AABBGeometry {
            center: center,
            extents: extents,
        }
    }

    pub fn overlaps(&self, ray: &Ray, mut t_min: f32, mut t_max: f32) -> bool {
        for axis in 0..3 {
            let inv_dir = 1.0 / ray.direction[axis];
            let mut t0 = (self.min()[axis] - ray.origin[axis]) * inv_dir;
            let mut t1 = (self.max()[axis] - ray.origin[axis]) * inv_dir;
            if inv_dir < 0.0 {
                let t_temp = t0;
                t0 = t1;
                t1 = t_temp;
            }
            t_min = if t0 > t_min { t0 } else { t_min };
            t_max = if t1 < t_max { t1 } else { t_max };
            if t_max <= t_min {
                return false;
            }
        }
        true
    }

    pub fn min(&self) -> Vec3 {
        self.center - self.extents.abs()
    }

    pub fn max(&self) -> Vec3 {
        self.center + self.extents.abs()
    }

    pub fn volume(&self) -> f32 {
        self.extents.x * self.extents.y * self.extents.z * 8.0
    }
}

impl Add for AABBGeometry {
    type Output = AABBGeometry;
    fn add(self, rhs: AABBGeometry) -> AABBGeometry {
        let min = self.min().min(&rhs.min());
        let max = self.max().max(&rhs.max());
        AABBGeometry::from_points(min, max)
    }
}

impl Add<&AABBGeometry> for AABBGeometry {
    type Output = AABBGeometry;
    fn add(self, rhs: &AABBGeometry) -> AABBGeometry {
        let min = self.min().min(&rhs.min());
        let max = self.max().max(&rhs.max());
        AABBGeometry::from_points(min, max)
    }
}

impl Add<AABBGeometry> for &AABBGeometry {
    type Output = AABBGeometry;
    fn add(self, rhs: AABBGeometry) -> AABBGeometry {
        let min = self.min().min(&rhs.min());
        let max = self.max().max(&rhs.max());
        AABBGeometry::from_points(min, max)
    }
}

impl Add<&AABBGeometry> for &AABBGeometry {
    type Output = AABBGeometry;
    fn add(self, rhs: &AABBGeometry) -> AABBGeometry {
        let min = self.min().min(&rhs.min());
        let max = self.max().max(&rhs.max());
        AABBGeometry::from_points(min, max)
    }
}

impl AddAssign<AABBGeometry> for AABBGeometry {
    fn add_assign(&mut self, rhs: AABBGeometry) {
        let min = self.min().min(&rhs.min());
        let max = self.max().max(&rhs.max());
        *self = AABBGeometry::from_points(min, max);
    }
}

impl AddAssign<&AABBGeometry> for AABBGeometry {
    fn add_assign(&mut self, rhs: &AABBGeometry) {
        let min = self.min().min(&rhs.min());
        let max = self.max().max(&rhs.max());
        *self = AABBGeometry::from_points(min, max);
    }
}

impl Add<Vec3> for AABBGeometry {
    type Output = AABBGeometry;
    fn add(self, rhs: Vec3) -> AABBGeometry {
        AABBGeometry {
            center: self.center + rhs,
            extents: self.extents,
        }
    }
}

impl Add<Vec3> for &AABBGeometry {
    type Output = AABBGeometry;
    fn add(self, rhs: Vec3) -> AABBGeometry {
        AABBGeometry {
            center: self.center + rhs,
            extents: self.extents,
        }
    }
}

impl Add<&Vec3> for AABBGeometry {
    type Output = AABBGeometry;
    fn add(self, rhs: &Vec3) -> AABBGeometry {
        AABBGeometry {
            center: self.center + rhs,
            extents: self.extents,
        }
    }
}

impl Add<&Vec3> for &AABBGeometry {
    type Output = AABBGeometry;
    fn add(self, rhs: &Vec3) -> AABBGeometry {
        AABBGeometry {
            center: self.center + rhs,
            extents: self.extents,
        }
    }
}

impl Mul<AABBGeometry> for Quaternion {
    type Output = AABBGeometry;
    fn mul(self, rhs: AABBGeometry) -> AABBGeometry {
        let mut min = Vec3::all(std::f32::INFINITY);
        let mut max = Vec3::all(std::f32::NEG_INFINITY);
        for x_sign in &[-1.0, 1.0] {
            for y_sign in &[-1.0, 1.0] {
                for z_sign in &[-1.0, 1.0] {
                    let offset = Vec3 {
                        x: x_sign * rhs.extents.x,
                        y: y_sign * rhs.extents.y,
                        z: z_sign * rhs.extents.z,
                    };
                    let rot_offset = self * offset + rhs.center;
                    min = min.min(&rot_offset);
                    max = max.max(&rot_offset);
                }
            }
        }
        AABBGeometry::from_points(min, max)
    }
}
