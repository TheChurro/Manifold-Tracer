use crate::geometry::sphere::{OrientedSpherePoint, SpherePoint};
use crate::geometry::spherinder::spaces::{SpherinderPoint, SpherinderTangent};

use na::{Vector2, Vector3};

pub struct Triangle<P, T> {
    pub a: SpherinderPoint<P>,
    pub b: SpherinderPoint<P>,
    pub c: SpherinderPoint<P>,
    pub normal: SpherinderTangent<T>,
}

impl Triangle<SpherePoint, Vector3<f32>> {
    pub fn new(
        a: SpherinderPoint<SpherePoint>,
        b: SpherinderPoint<SpherePoint>,
        c: SpherinderPoint<SpherePoint>,
    ) -> Triangle<SpherePoint, Vector3<f32>> {
        let ab_perp = a.sphere.cross(&b.sphere.xyz());
        let ab_end = ab_perp.cross(&a.sphere).normalize();
        let ab_end_dw =
            (b.depth - a.depth) / (a.sphere.dot(&b.sphere)).acos() * std::f32::consts::PI;
        let ac_perp = a.sphere.cross(&c.sphere).normalize();
        let ac_end = ac_perp.cross(&a.sphere);
        let ac_end_dw =
            (c.depth - a.depth) / (a.sphere.dot(&c.sphere)).acos() * std::f32::consts::PI;
        let tangent_d = ab_end.cross(&a.sphere).normalize();
        let ab_tan = Vector3::new(1.0, 0.0, ab_end_dw);
        let ac_tan = Vector3::new(ab_end.dot(&ac_end), tangent_d.dot(&ac_end), ac_end_dw);
        let normal_tan = ab_tan.cross(&ac_tan).normalize();
        let normal_sphere: Vector3<f32> = ab_end * normal_tan.x + tangent_d * normal_tan.y;
        Triangle {
            a: a,
            b: b,
            c: c,
            normal: (normal_sphere, normal_tan.z).into(),
        }
    }
}

impl Triangle<OrientedSpherePoint, Vector2<f32>> {
    pub fn new(
        a: SpherinderPoint<OrientedSpherePoint>,
        b: SpherinderPoint<OrientedSpherePoint>,
        c: SpherinderPoint<OrientedSpherePoint>,
    ) -> Triangle<OrientedSpherePoint, Vector2<f32>> {
        let a_inv = a.sphere.inverse();
        let b_rot = (b.sphere * a_inv) * Vector3::x_axis();
        let c_rot = (c.sphere * a_inv) * Vector3::x_axis();
        let ab_tan = Vector3::new(b_rot.z, b_rot.y, b.depth - a.depth);
        let ac_tan = Vector3::new(c_rot.z, c_rot.y, c.depth - a.depth);
        let normal_tan = ab_tan.cross(&ac_tan).normalize();
        Triangle {
            a: a,
            b: b,
            c: c,
            normal: (normal_tan.xy(), normal_tan.z).into(),
        }
    }
}

pub trait Triangulable<P, T> : Sized {
    fn triangle(self, b: Self, c: Self) -> Triangle<P, T>;
    fn triangle_new(&self, b: &Self, c: &Self) -> Triangle<P, T>;
}

impl Triangulable<SpherePoint, Vector3<f32>> for SpherinderPoint<SpherePoint> {
    fn triangle(self, b: Self, c: Self) -> Triangle<SpherePoint, Vector3<f32>> {
        Triangle::<SpherePoint, Vector3<f32>>::new(self, b, c)
    }
    fn triangle_new(&self, b: &Self, c: &Self) -> Triangle<SpherePoint, Vector3<f32>> {
        Triangle::<SpherePoint, Vector3<f32>>::new(self.clone(), b.clone(), c.clone())
    }
}

impl Triangulable<OrientedSpherePoint, Vector2<f32>> for SpherinderPoint<OrientedSpherePoint> {
    fn triangle(self, b: Self, c: Self) -> Triangle<OrientedSpherePoint, Vector2<f32>> {
        Triangle::<OrientedSpherePoint, Vector2<f32>>::new(self, b, c)
    }
    fn triangle_new(&self, b: &Self, c: &Self) -> Triangle<OrientedSpherePoint, Vector2<f32>> {
        Triangle::<OrientedSpherePoint, Vector2<f32>>::new(self.clone(), b.clone(), c.clone())
    }
}
