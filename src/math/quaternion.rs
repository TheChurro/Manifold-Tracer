use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

use crate::math::vectors::Vec3;

#[derive(Clone, Copy, Debug)]
pub struct Quaternion {
    pub a: f32,
    pub b: f32,
    pub c: f32,
    pub d: f32,
}

impl Quaternion {
    pub fn identity() -> Quaternion {
        Quaternion {
            a: 1.0,
            b: 0.0,
            c: 0.0,
            d: 0.0,
        }
    }
    pub fn i() -> Quaternion {
        Quaternion {
            a: 0.0,
            b: 1.0,
            c: 0.0,
            d: 0.0,
        }
    }
    pub fn j() -> Quaternion {
        Quaternion {
            a: 0.0,
            b: 0.0,
            c: 1.0,
            d: 0.0,
        }
    }
    pub fn k() -> Quaternion {
        Quaternion {
            a: 0.0,
            b: 0.0,
            c: 0.0,
            d: 1.0,
        }
    }
    pub fn new(a: f32, b: f32, c: f32, d: f32) -> Quaternion {
        Quaternion {
            a: a,
            b: b,
            c: c,
            d: d,
        }
    }
    pub fn conj(&self) -> Quaternion {
        Quaternion {
            a: self.a,
            b: -self.b,
            c: -self.c,
            d: -self.d,
        }
    }
    pub fn length_sq(&self) -> f32 {
        self.a * self.a + self.b * self.b + self.c * self.c + self.d * self.d
    }
    pub fn length(&self) -> f32 {
        self.length_sq().sqrt()
    }
    pub fn inv(&self) -> Quaternion {
        self.conj() / self.length_sq()
    }
    pub fn axis_angle(axis: Vec3, angle: f32) -> Quaternion {
        let sine = (angle / 2.0).sin();
        Quaternion {
            a: (angle / 2.0).cos(),
            b: axis.x * sine,
            c: axis.y * sine,
            d: axis.z * sine,
        }
    }
    pub fn look_at(forward: Vec3, up: Vec3) -> Quaternion {
        let side = up.cross(&forward).normalized();
        let actual_up = forward.cross(&side);
        let trace = side.x + actual_up.y + forward.z;
        let descriminant = 1.0 + trace;
        if descriminant < 0.0001 {
            let x_largest = side.x > actual_up.y && side.x > forward.z;
            let y_largest = actual_up.y > side.x && actual_up.y > forward.z;
            if x_largest {
                let r = (1.0 + side.x - actual_up.y - forward.z).sqrt();
                let s = 1.0 / (2.0 * r);
                Quaternion::new(
                    s * (actual_up.z - forward.y),
                    0.5 * r,
                    s * (actual_up.x + side.y),
                    s * (forward.x + side.z),
                )
            } else if y_largest {
                let r = (1.0 - side.x + actual_up.y - forward.z).sqrt();
                let s = 1.0 / (2.0 * r);
                Quaternion::new(
                    s * (forward.x - side.y),
                    s * (actual_up.x + side.y),
                    0.5 * r,
                    s * (forward.y + actual_up.z),
                )
            } else {
                let r = (1.0 - side.x - actual_up.y + forward.z).sqrt();
                let s = 1.0 / (2.0 * r);
                Quaternion::new(
                    s * (side.y - actual_up.x),
                    s * (forward.x + side.z),
                    s * (forward.y + actual_up.z),
                    0.5 * r,
                )
            }
        } else {
            let r = descriminant.sqrt();
            let s = 1.0 / (2.0 * r);
            Quaternion::new(
                0.5 * r,
                s * (actual_up.z - forward.y),
                s * (forward.x - side.z),
                s * (side.y - actual_up.x),
            )
        }
    }
}

op_impl!(Quaternion, Add, add, a, b, c, d);
op_assign_impl!(Quaternion, AddAssign, add_assign, a, b, c, d);
op_impl!(Quaternion, Sub, sub, a, b, c, d);
op_assign_impl!(Quaternion, SubAssign, sub_assign, a, b, c, d);
op_scalar_impl!(Quaternion, f32, Mul, mul, a, b, c, d);
op_scalar_assign_impl!(Quaternion, f32, MulAssign, mul_assign, a, b, c, d);
op_scalar_impl!(Quaternion, f32, Div, div, a, b, c, d);
op_scalar_assign_impl!(Quaternion, f32, DivAssign, div_assign, a, b, c, d);

impl Mul for Quaternion {
    type Output = Quaternion;
    fn mul(self, rhs: Quaternion) -> Quaternion {
        Quaternion {
            a: self.a * rhs.a - self.b * rhs.b - self.c * rhs.c - self.d * rhs.d,
            b: self.a * rhs.b + self.b * rhs.a + self.c * rhs.d - self.d * rhs.c,
            c: self.a * rhs.c - self.b * rhs.d + self.c * rhs.a + self.d * rhs.b,
            d: self.a * rhs.d + self.b * rhs.c - self.c * rhs.b + self.d * rhs.a,
        }
    }
}

impl Mul<&Quaternion> for &Quaternion {
    type Output = Quaternion;
    fn mul(self, rhs: &Quaternion) -> Quaternion {
        Quaternion {
            a: self.a * rhs.a - self.b * rhs.b - self.c * rhs.c - self.d * rhs.d,
            b: self.a * rhs.b + self.b * rhs.a + self.c * rhs.d - self.d * rhs.c,
            c: self.a * rhs.c - self.b * rhs.d + self.c * rhs.a + self.d * rhs.b,
            d: self.a * rhs.d + self.b * rhs.c - self.c * rhs.b + self.d * rhs.a,
        }
    }
}

impl Mul<Quaternion> for &Quaternion {
    type Output = Quaternion;
    fn mul(self, rhs: Quaternion) -> Quaternion {
        Quaternion {
            a: self.a * rhs.a - self.b * rhs.b - self.c * rhs.c - self.d * rhs.d,
            b: self.a * rhs.b + self.b * rhs.a + self.c * rhs.d - self.d * rhs.c,
            c: self.a * rhs.c - self.b * rhs.d + self.c * rhs.a + self.d * rhs.b,
            d: self.a * rhs.d + self.b * rhs.c - self.c * rhs.b + self.d * rhs.a,
        }
    }
}

impl Mul<&Quaternion> for Quaternion {
    type Output = Quaternion;
    fn mul(self, rhs: &Quaternion) -> Quaternion {
        Quaternion {
            a: self.a * rhs.a - self.b * rhs.b - self.c * rhs.c - self.d * rhs.d,
            b: self.a * rhs.b + self.b * rhs.a + self.c * rhs.d - self.d * rhs.c,
            c: self.a * rhs.c - self.b * rhs.d + self.c * rhs.a + self.d * rhs.b,
            d: self.a * rhs.d + self.b * rhs.c - self.c * rhs.b + self.d * rhs.a,
        }
    }
}

impl MulAssign for Quaternion {
    fn mul_assign(&mut self, rhs: Quaternion) {
        *self = Quaternion {
            a: self.a * rhs.a - self.b * rhs.b - self.c * rhs.c - self.d * rhs.d,
            b: self.a * rhs.b + self.b * rhs.a + self.c * rhs.d - self.d * rhs.c,
            c: self.a * rhs.c - self.b * rhs.d + self.c * rhs.a + self.d * rhs.b,
            d: self.a * rhs.d + self.b * rhs.c - self.c * rhs.b + self.d * rhs.a,
        }
    }
}

impl Mul<Vec3> for Quaternion {
    type Output = Vec3;
    fn mul(self, rhs: Vec3) -> Vec3 {
        let rhs_quat = Quaternion::new(0.0, rhs.x, rhs.y, rhs.z);
        let out_quat = self * rhs_quat * self.inv();
        Vec3::new(out_quat.b, out_quat.c, out_quat.d)
    }
}

impl Mul<&Vec3> for &Quaternion {
    type Output = Vec3;
    fn mul(self, rhs: &Vec3) -> Vec3 {
        let rhs_quat = Quaternion::new(0.0, rhs.x, rhs.y, rhs.z);
        let out_quat = self * rhs_quat * self.inv();
        Vec3::new(out_quat.b, out_quat.c, out_quat.d)
    }
}

impl Mul<&Vec3> for Quaternion {
    type Output = Vec3;
    fn mul(self, rhs: &Vec3) -> Vec3 {
        let rhs_quat = Quaternion::new(0.0, rhs.x, rhs.y, rhs.z);
        let out_quat = self * rhs_quat * self.inv();
        Vec3::new(out_quat.b, out_quat.c, out_quat.d)
    }
}

impl Mul<Vec3> for &Quaternion {
    type Output = Vec3;
    fn mul(self, rhs: Vec3) -> Vec3 {
        let rhs_quat = Quaternion::new(0.0, rhs.x, rhs.y, rhs.z);
        let out_quat = self * rhs_quat * self.inv();
        Vec3::new(out_quat.b, out_quat.c, out_quat.d)
    }
}
