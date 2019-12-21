use crate::math::geometry::aabb::AABBGeometry;
use crate::math::geometry::rect::RectGeometry;
use crate::math::geometry::sphere::SphereGeometry;
use crate::math::geometry::volumes::ConstantVolume;
use crate::math::quaternion::Quaternion;
use crate::math::ray::{Ray, RayCollidable, RayHit};
use crate::math::vectors::Vec3;

pub enum Collider {
    Sphere(SphereGeometry),
    SphereWithVelocity(SphereGeometry, Vec3),
    Rect(RectGeometry),
    Volume(ConstantVolume),
    Translate(Vec3, Box<Collider>),
    Rotate(Quaternion, Box<Collider>),
    Union(Vec<Collider>),
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
            &Rect(ref geometry) => geometry.bounding_box(t_min, t_max),
            &Volume(ref volume) => volume.bounding_box(t_min, t_max),
            &Translate(offset, ref collider) => {
                collider.bounding_box(t_min, t_max).map(|x| x + offset)
            }
            &Rotate(rotation, ref collider) => {
                collider.bounding_box(t_min, t_max).map(|x| rotation * x)
            }
            &Union(ref colliders) => {
                if colliders.len() == 0 {
                    None
                } else {
                    let mut aabb = None;
                    for collider in colliders {
                        if let Some(new) = collider.bounding_box(t_min, t_max) {
                            if aabb.is_none() {
                                aabb = Some(new);
                            } else {
                                aabb = aabb.map(|x| x + new);
                            }
                        }
                    }
                    aabb
                }
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
            &Rect(ref geometry) => geometry.hit(ray, t_min, t_max),
            &Volume(ref volume) => volume.hit(ray, t_min, t_max),
            &Translate(offset, ref collider) => {
                let offset_ray = Ray {
                    cast_time: ray.cast_time,
                    origin: ray.origin - offset,
                    direction: ray.direction,
                };
                if let Some(mut hit) = collider.hit(&offset_ray, t_min, t_max) {
                    hit.location += offset;
                    Some(hit)
                } else {
                    None
                }
            }
            &Rotate(rotation, ref collider) => {
                let inv_rotation = rotation.inv();
                let offset_ray = Ray {
                    cast_time: ray.cast_time,
                    origin: inv_rotation * ray.origin,
                    direction: inv_rotation * ray.direction,
                };
                if let Some(mut hit) = collider.hit(&offset_ray, t_min, t_max) {
                    hit.location = rotation * hit.location;
                    hit.normal = rotation * hit.normal;
                    Some(hit)
                } else {
                    None
                }
            }
            &Union(ref colliders) => {
                let mut best_hit: Option<RayHit> = None;
                let mut earliest_time = t_max;
                for collider in colliders {
                    if let Some(hit) = collider.hit(&ray, t_min, earliest_time) {
                        earliest_time = hit.hit_fraction;
                        best_hit = Some(hit);
                    }
                }
                if best_hit.is_some() {}
                best_hit
            }
        }
    }
}

impl From<SphereGeometry> for Collider {
    fn from(geometry: SphereGeometry) -> Collider {
        Collider::Sphere(geometry)
    }
}

impl From<RectGeometry> for Collider {
    fn from(geometry: RectGeometry) -> Collider {
        Collider::Rect(geometry)
    }
}

impl Collider {
    pub fn with_velocity(self, velocity: Vec3) -> Collider {
        use Collider::*;
        match self {
            Sphere(geometry) => SphereWithVelocity(geometry, velocity),
            SphereWithVelocity(geometry, _) => SphereWithVelocity(geometry, velocity),
            Rect(geometry) => Rect(geometry),
            Volume(vol) => Volume(vol),
            Translate(offset, collider) => Translate(offset, collider),
            Rotate(rotation, collider) => Rotate(rotation, collider),
            Union(colliders) => Union(colliders),
        }
    }

    pub fn translate(self, offset: Vec3) -> Collider {
        Collider::Translate(offset, Box::new(self))
    }

    pub fn rotate(self, rotation: Quaternion) -> Collider {
        Collider::Rotate(rotation, Box::new(self))
    }

    pub fn to_volume(self, density: f32) -> Collider {
        Collider::Volume(ConstantVolume {
            boundary: Box::new(self),
            density: density,
        })
    }
}
