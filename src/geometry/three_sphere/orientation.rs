use crate::geometry::three_sphere::representation::*;

use na::{Quaternion, UnitQuaternion, Vector3};

#[derive(Clone, Copy)]
pub struct Orientation {
    pub left_isoclinic: UnitQuaternion<f32>,
    pub right_isoclinic: UnitQuaternion<f32>,
}

impl Orientation {
    pub fn identity() -> Orientation {
        Orientation {
            left_isoclinic: UnitQuaternion::identity(),
            right_isoclinic: UnitQuaternion::identity(),
        }
    }

    pub fn rotate_towards(
        mut imaginary: Direction,
        by_angle: f32,
        contra_angle: f32,
    ) -> Orientation {
        let left_angle = (by_angle - contra_angle) / 2.0;
        let sine_left = left_angle.sin();
        let right_angle = (by_angle + contra_angle) / 2.0;
        let sine_right = right_angle.sin();
        let imaginary_norm = imaginary.length();
        if imaginary_norm > 0.0001 {
            imaginary = imaginary * (1.0 / imaginary_norm);
        }
        Orientation {
            left_isoclinic: UnitQuaternion::new_normalize(Quaternion::new(
                left_angle.cos(),
                sine_left * imaginary.i,
                sine_left * imaginary.j,
                sine_left * imaginary.k,
            )),
            right_isoclinic: UnitQuaternion::new_normalize(Quaternion::new(
                right_angle.cos(),
                sine_right * imaginary.i,
                sine_right * imaginary.j,
                sine_right * imaginary.k,
            )),
        }
    }

    pub fn rotate_on_plane(a: &Point, b: &Point, angle: f32, contra_angle: f32) -> Orientation {
        // l is a unit quaternion which rotates a to b on the left. This means,
        // l * a = b. We can see this by the fact that l * a = (b * a^{-1}) * a = b * (a^{-1}a) = b
        let l = (**b) * a.inverse();
        // r is a unit quaternion which rotates a to b on the right. This means a * r = b.
        let r = a.inverse() * (**b);
        // Next, we compute the rotation being done by l. It will take the form of
        // cos(theta) + sin(theta) v for some pure imaginary quaternion v.
        let imaginary_l = Vector3::new(l.i, l.j, l.k);
        let base_angle_l = imaginary_l.magnitude().atan2(l.w);
        // Then, we compute the rotation being done by r.
        let imaginary_r = Vector3::new(r.i, r.j, r.k);
        let base_angle_r = imaginary_r.magnitude().atan2(r.w);
        // If the base angle is 0, then l is the identity. So a = b and we cannot perform
        // any meaningful rotation.
        if base_angle_l.abs() < EPSILON {
            Orientation::identity()
        } else {
            // Because l and r induce isoclinic rotation, the rotation angles base_angle_l and
            // base_angle_r are both ±acos(a.dot(b)). We want them to be respectively
            // (angle + contra_angle) / 2 and (angle - contra_angle) / 2 to properly rotate
            // on the plane spanned by a and b.
            let new_base_angle_l = (angle + contra_angle) / 2.0;
            let new_imaginary_l = new_base_angle_l.sin() * imaginary_l / base_angle_l.sin();
            let new_base_angle_r = (angle - contra_angle) / 2.0;
            let new_imaginary_r = new_base_angle_r.sin() * imaginary_r / base_angle_r.sin();
            let new_rotation_l = UnitQuaternion::new_normalize(Quaternion::new(
                new_base_angle_l.cos(),
                new_imaginary_l.x,
                new_imaginary_l.y,
                new_imaginary_l.z,
            ));
            let new_rotation_r = UnitQuaternion::new_normalize(Quaternion::new(
                new_base_angle_r.cos(),
                new_imaginary_r.x,
                new_imaginary_r.y,
                new_imaginary_r.z,
            ));
            Orientation {
                left_isoclinic: new_rotation_l,
                right_isoclinic: new_rotation_r,
            }
        }
    }
}

use std::ops::Mul;
impl Mul<Point> for Orientation {
    type Output = Point;
    fn mul(self, p: Point) -> Point {
        Point(self.left_isoclinic * (*p) * self.right_isoclinic)
    }
}
impl Mul<Direction> for Orientation {
    type Output = Direction;
    fn mul(self, d: Direction) -> Direction {
        Direction(*self.left_isoclinic * (*d) * *self.right_isoclinic)
    }
}
impl Mul<Orientation> for Orientation {
    type Output = Orientation;
    fn mul(self, d: Orientation) -> Orientation {
        Orientation {
            left_isoclinic: self.left_isoclinic * d.left_isoclinic,
            right_isoclinic: d.right_isoclinic * self.right_isoclinic,
        }
    }
}

impl Mul<Point> for &Orientation {
    type Output = Point;
    fn mul(self, p: Point) -> Point {
        Point(self.left_isoclinic * *p * self.right_isoclinic)
    }
}
impl Mul<Direction> for &Orientation {
    type Output = Direction;
    fn mul(self, d: Direction) -> Direction {
        Direction(*self.left_isoclinic * *d * *self.right_isoclinic)
    }
}
impl Mul<Orientation> for &Orientation {
    type Output = Orientation;
    fn mul(self, d: Orientation) -> Orientation {
        Orientation {
            left_isoclinic: self.left_isoclinic * d.left_isoclinic,
            right_isoclinic: d.right_isoclinic * self.right_isoclinic,
        }
    }
}
