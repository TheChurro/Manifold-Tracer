use crate::geometry::space::manifold::Manifold;

use na::{Unit, UnitQuaternion, Vector2, Vector3};

pub struct Sphere {}
pub type SpherePoint = Unit<Vector3<f32>>;
pub type OrientedSpherePoint = UnitQuaternion<f32>;
pub type Normal3 = Unit<Vector3<f32>>;

pub trait OntoSphere {
    fn point(&self) -> SpherePoint;
}

impl OntoSphere for SpherePoint {
    fn point(&self) -> SpherePoint {
        *self
    }
}

impl OntoSphere for OrientedSpherePoint {
    fn point(&self) -> SpherePoint {
        self * Vector3::x_axis()
    }
}

pub struct GreatCircle<T> {
    pub base: T,
    pub normal: Normal3,
    pub rate: f32,
}

pub struct GreatArc<T> {
    pub circle: GreatCircle<T>,
    pub max_angle: f32,
}

impl
    Manifold<
        OrientedSpherePoint,
        Vector2<f32>,
        GreatCircle<OrientedSpherePoint>,
        GreatArc<OrientedSpherePoint>,
    > for Sphere
{
    fn exponential(
        &self,
        point: OrientedSpherePoint,
        tangent: Vector2<f32>,
    ) -> GreatCircle<OrientedSpherePoint> {
        let tangent = point * Vector3::new(0.0, tangent.y, tangent.x);
        GreatCircle {
            base: point,
            normal: Unit::new_normalize((point * Vector3::x_axis()).cross(&tangent)),
            rate: tangent.norm(),
        }
    }

    fn logarithmic(
        &self,
        a: OrientedSpherePoint,
        b: OrientedSpherePoint,
    ) -> GreatCircle<OrientedSpherePoint> {
        let rotation = a.rotation_to(&b);
        GreatCircle {
            base: a,
            normal: rotation.axis().unwrap_or(Vector3::z_axis()),
            rate: rotation.angle(),
        }
    }

    fn set_length(
        &self,
        start: OrientedSpherePoint,
        length: f32,
        mut g: GreatCircle<OrientedSpherePoint>,
    ) -> GreatArc<OrientedSpherePoint> {
        g.base = start;
        GreatArc {
            circle: g,
            max_angle: length,
        }
    }

    fn shorten(
        &self,
        g: GreatArc<OrientedSpherePoint>,
        new_length: f32,
    ) -> GreatArc<OrientedSpherePoint> {
        GreatArc {
            circle: g.circle,
            max_angle: new_length,
        }
    }

    fn segment(
        &self,
        a: OrientedSpherePoint,
        b: OrientedSpherePoint,
        mut g: GreatCircle<OrientedSpherePoint>,
    ) -> GreatArc<OrientedSpherePoint> {
        g.base = a;
        GreatArc {
            circle: g,
            max_angle: a.angle_to(&b),
        }
    }

    fn eval_length(&self, g: GreatCircle<OrientedSpherePoint>, length: f32) -> OrientedSpherePoint {
        let rotation = UnitQuaternion::from_axis_angle(&g.normal, length);
        rotation * g.base
    }

    fn eval_time(&self, g: GreatCircle<OrientedSpherePoint>, time: f32) -> OrientedSpherePoint {
        let distance = g.rate * time;
        self.eval_length(g, distance)
    }
}

impl Manifold<SpherePoint, Vector3<f32>, GreatCircle<SpherePoint>, GreatArc<SpherePoint>>
    for Sphere
{
    fn exponential(&self, point: SpherePoint, tangent: Vector3<f32>) -> GreatCircle<SpherePoint> {
        let tangent = tangent - point.dot(&tangent) * point.into_inner();
        GreatCircle {
            base: point,
            normal: Unit::new_normalize(tangent),
            rate: tangent.norm(),
        }
    }

    fn logarithmic(&self, a: SpherePoint, b: SpherePoint) -> GreatCircle<SpherePoint> {
        GreatCircle {
            base: a,
            normal: Unit::new_normalize(a.cross(&b)),
            rate: a.dot(&b).acos(),
        }
    }

    fn set_length(
        &self,
        start: SpherePoint,
        length: f32,
        mut g: GreatCircle<SpherePoint>,
    ) -> GreatArc<SpherePoint> {
        g.base = start;
        GreatArc {
            circle: g,
            max_angle: length,
        }
    }

    fn shorten(&self, g: GreatArc<SpherePoint>, new_length: f32) -> GreatArc<SpherePoint> {
        GreatArc {
            circle: g.circle,
            max_angle: new_length,
        }
    }

    fn segment(
        &self,
        a: SpherePoint,
        b: SpherePoint,
        mut g: GreatCircle<SpherePoint>,
    ) -> GreatArc<SpherePoint> {
        g.base = a;
        GreatArc {
            circle: g,
            max_angle: a.dot(&b).acos(),
        }
    }

    fn eval_length(&self, g: GreatCircle<SpherePoint>, length: f32) -> SpherePoint {
        let rotation = UnitQuaternion::from_axis_angle(&g.normal, length);
        rotation * g.base
    }

    fn eval_time(&self, g: GreatCircle<SpherePoint>, time: f32) -> SpherePoint {
        let rate = g.rate;
        self.eval_length(g, rate * time)
    }
}
