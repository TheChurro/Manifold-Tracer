use crate::math::geometry::aabb::AABBGeometry;
use crate::math::geometry::sphere::SphereGeometry;
use crate::math::ray::{Ray, RayCollidable, RayHit};
use crate::math::vectors::Vec3;

pub enum Collider {
    Sphere(SphereGeometry),
    SphereWithVelocity(SphereGeometry, Vec3),
}

impl RayCollidable for Collider {
    fn bounding_box(&self, t_min: f32, t_max: f32) -> Option<AABBGeometry> {
        use Collider::*;
        match self {
            &Sphere(ref geometry) => geometry.bounding_box(t_min, t_max),
            &SphereWithVelocity(ref geometry, ref velocity) => {
                // Sphere geometry always returns a bounding box so we are good to simply add them
                let begin_aabb = geometry.offset(velocity * t_min).bounding_box(t_min, t_max);
                let end_aabb = geometry.offset(velocity * t_max).bounding_box(t_min, t_max);
                Some(begin_aabb.unwrap() + end_aabb.unwrap())
            }
        }
    }

    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<RayHit> {
        use Collider::*;
        match self {
            &Sphere(ref geometry) => geometry.hit(ray, t_min, t_max),
            &SphereWithVelocity(ref geometry, ref velocity) => geometry
                .offset(velocity * ray.cast_time)
                .hit(ray, t_min, t_max),
        }
    }
}

impl Collider {
    pub fn new(geometry: SphereGeometry) -> Collider {
        Collider::Sphere(geometry)
    }

    pub fn with_velocity(self, velocity: Vec3) -> Collider {
        use Collider::*;
        match self {
            Sphere(geometry) => SphereWithVelocity(geometry, velocity),
            SphereWithVelocity(geometry, _) => SphereWithVelocity(geometry, velocity),
        }
    }
}
