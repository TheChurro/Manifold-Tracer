use crate::geometry::space::manifold::Manifold;
use crate::geometry::sphere::*;

use std::marker::PhantomData;

use na::Vector3;

pub struct Spherinder<P, T>(PhantomData<P>, PhantomData<T>);
impl<P, T> Spherinder<P, T> {
    pub fn new() -> Self {
        Spherinder(PhantomData, PhantomData)
    }
}

#[derive(Clone, Copy)]
pub struct SpherinderPoint<P> {
    pub sphere: P,
    pub depth: f32,
}

impl<P> From<(P, f32)> for SpherinderPoint<P> {
    fn from(coords: (P, f32)) -> SpherinderPoint<P> {
        SpherinderPoint {
            sphere: coords.0,
            depth: coords.1,
        }
    }
}

#[derive(Clone, Copy)]
pub struct SpherinderTangent<T> {
    pub sphere: T,
    pub depth: f32,
}

impl<T> From<(T, f32)> for SpherinderTangent<T> {
    fn from(coords: (T, f32)) -> SpherinderTangent<T> {
        SpherinderTangent {
            sphere: coords.0,
            depth: coords.1,
        }
    }
}

pub trait SquaredNorm {
    fn squared_norm(&self) -> f32;
}

impl SquaredNorm for Vector3<f32> {
    fn squared_norm(&self) -> f32 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }
}

impl SquaredNorm for na::Vector2<f32> {
    fn squared_norm(&self) -> f32 {
        self.x * self.x + self.y * self.y
    }
}

impl<T: SquaredNorm> SpherinderTangent<T> {
    pub fn norm(&self) -> f32 {
        (self.sphere.squared_norm() + self.depth + self.depth).sqrt()
    }
}

pub struct SpherinderGeodesic<T> {
    pub sphere_geodesic: GreatCircle<T>,
    pub depth_start: f32,
    pub depth_rate: f32,
}

pub struct SpherinderGeodesicSegment<T> {
    pub sphere_segment: GreatArc<T>,
    pub depth_start: f32,
    pub depth_max: f32,
    pub depth_rate: f32,
}

impl<P, T>
    Manifold<
        SpherinderPoint<P>,
        SpherinderTangent<T>,
        SpherinderGeodesic<P>,
        SpherinderGeodesicSegment<P>,
    > for Spherinder<P, T>
where
    Sphere: Manifold<P, T, GreatCircle<P>, GreatArc<P>>,
{
    fn exponential(
        &self,
        point: SpherinderPoint<P>,
        tangent: SpherinderTangent<T>,
    ) -> SpherinderGeodesic<P> {
        let sphere_geodesic = Sphere {}.exponential(point.sphere, tangent.sphere);
        SpherinderGeodesic {
            sphere_geodesic: sphere_geodesic,
            depth_start: point.depth,
            depth_rate: tangent.depth,
        }
    }

    fn logarithmic(&self, a: SpherinderPoint<P>, b: SpherinderPoint<P>) -> SpherinderGeodesic<P> {
        SpherinderGeodesic {
            sphere_geodesic: Sphere {}.logarithmic(a.sphere, b.sphere),
            depth_start: a.depth,
            depth_rate: b.depth - a.depth,
        }
    }

    fn set_length(
        &self,
        start: SpherinderPoint<P>,
        length: f32,
        g: SpherinderGeodesic<P>,
    ) -> SpherinderGeodesicSegment<P> {
        let phi = g.sphere_geodesic.rate.atan2(g.depth_rate);
        let sphere_length = length * phi.sin();
        let depth_length = length * phi.cos();
        SpherinderGeodesicSegment {
            sphere_segment: Sphere {}.set_length(start.sphere, sphere_length, g.sphere_geodesic),
            depth_start: start.depth,
            depth_rate: g.depth_rate,
            depth_max: g.depth_start + depth_length,
        }
    }

    fn shorten(
        &self,
        g: SpherinderGeodesicSegment<P>,
        new_length: f32,
    ) -> SpherinderGeodesicSegment<P> {
        let phi = g.sphere_segment.circle.rate.atan2(g.depth_rate);
        let sphere_length = new_length * phi.sin();
        let depth_length = new_length * phi.cos();
        SpherinderGeodesicSegment {
            sphere_segment: Sphere {}.shorten(g.sphere_segment, sphere_length),
            depth_start: g.depth_start,
            depth_rate: g.depth_rate,
            depth_max: g.depth_start + depth_length,
        }
    }

    fn segment(
        &self,
        a: SpherinderPoint<P>,
        b: SpherinderPoint<P>,
        g: SpherinderGeodesic<P>,
    ) -> SpherinderGeodesicSegment<P> {
        SpherinderGeodesicSegment {
            sphere_segment: Sphere {}.segment(a.sphere, b.sphere, g.sphere_geodesic),
            depth_start: a.depth,
            depth_rate: g.depth_start,
            depth_max: b.depth,
        }
    }

    fn eval_length(&self, g: SpherinderGeodesic<P>, length: f32) -> SpherinderPoint<P> {
        let phi = g.sphere_geodesic.rate.atan2(g.depth_rate);
        let sphere_length = length * phi.sin();
        let depth_length = if g.depth_rate < 0.0 {
            -length * phi.cos()
        } else {
            length * phi.cos()
        };
        SpherinderPoint {
            sphere: Sphere {}.eval_length(g.sphere_geodesic, sphere_length),
            depth: g.depth_start + depth_length,
        }
    }

    fn eval_time(&self, g: SpherinderGeodesic<P>, time: f32) -> SpherinderPoint<P> {
        SpherinderPoint {
            sphere: Sphere {}.eval_time(g.sphere_geodesic, time),
            depth: g.depth_start + g.depth_rate * time,
        }
    }
}
