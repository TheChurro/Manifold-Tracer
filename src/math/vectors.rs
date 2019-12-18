use std::f32;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};
use std::ops::Index;

#[derive(Clone, Copy, Debug)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

op_impl!(Vec3, Add, add, x, y, z);
op_assign_impl!(Vec3, AddAssign, add_assign, x, y, z);
op_impl!(Vec3, Sub, sub, x, y, z);
op_assign_impl!(Vec3, SubAssign, sub_assign, x, y, z);
op_scalar_impl!(Vec3, f32, Mul, mul, x, y, z);
op_scalar_assign_impl!(Vec3, f32, MulAssign, mul_assign, x, y, z);
op_scalar_impl!(Vec3, f32, Div, div, x, y, z);
op_scalar_assign_impl!(Vec3, f32, DivAssign, div_assign, x, y, z);

impl Vec3 {
    pub fn new(x: f32, y: f32, z: f32) -> Vec3 {
        Vec3 { x: x, y: y, z: z }
    }

    pub fn all(v: f32) -> Vec3 {
        Vec3 { x: v, y: v, z: v }
    }

    pub fn dot(&self, other: &Vec3) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn cross(&self, other: &Vec3) -> Vec3 {
        Vec3 {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }

    pub fn length(&self) -> f32 {
        f32::sqrt(self.length_sq())
    }

    pub fn length_sq(&self) -> f32 {
        self.dot(self)
    }

    pub fn abs(&self) -> Vec3 {
        Vec3 {
            x: self.x.abs(),
            y: self.y.abs(),
            z: self.z.abs(),
        }
    }

    pub fn min(&self, rhs: &Vec3) -> Vec3 {
        Vec3 {
            x: self.x.min(rhs.x),
            y: self.y.min(rhs.y),
            z: self.z.min(rhs.z),
        }
    }

    pub fn max(&self, rhs: &Vec3) -> Vec3 {
        Vec3 {
            x: self.x.max(rhs.x),
            y: self.y.max(rhs.y),
            z: self.z.max(rhs.z),
        }
    }

    pub fn zero() -> Vec3 {
        Vec3 {
            x: 0f32,
            y: 0f32,
            z: 0f32,
        }
    }

    pub fn left() -> Vec3 {
        Vec3 {
            x: 1f32,
            y: 0f32,
            z: 0f32,
        }
    }

    pub fn up() -> Vec3 {
        Vec3 {
            x: 0f32,
            y: 1f32,
            z: 0f32,
        }
    }

    pub fn forward() -> Vec3 {
        Vec3 {
            x: 0f32,
            y: 0f32,
            z: 1f32,
        }
    }

    pub fn normalized(&self) -> Vec3 {
        let norm = self.length();
        if norm < 0.00001 {
            Self::zero()
        } else {
            self / norm
        }
    }
}

impl Index<usize> for Vec3 {
    type Output = f32;
    fn index(&self, index: usize) -> &f32 {
        match index {
            0 => &self.x,
            1 => &self.y,
            2 => &self.z,
            _ => { panic!("Invalid index for vector!"); }
        }
    }
}
