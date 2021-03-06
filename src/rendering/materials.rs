use rand::distributions::Uniform;
use rand::Rng;
use rand_distr::{Distribution, UnitBall, UnitSphere};

use crate::math::colors::Color;
use crate::math::quaternion::Quaternion;
use crate::math::ray::{Ray, RayHit};
use crate::math::vectors::Vec3;

use crate::rendering::textures::{TextureAtlas, TextureIndex};

#[derive(Copy, Clone, Debug)]
pub enum Material {
    Lambertian {
        albedo: TextureIndex,
    },
    Metal {
        albedo: TextureIndex,
        fuzziness: f32,
    },
    Dielectric {
        index_of_refraction: f32,
    },
    Emissive {
        texture: TextureIndex,
        amplify: f32,
    },
    Isotropic {
        albedo: TextureIndex,
    },
    Projective,
    DirectionCollapse
}

pub fn reflect(target: Vec3, normal: Vec3) -> Vec3 {
    target - 2.0 * target.dot(&normal) * normal
}

pub fn refract(direction: Vec3, normal: Vec3, ni_over_nt: f32) -> Option<Vec3> {
    let perp = direction.dot(&normal);
    let descriminant = 1.0 - ni_over_nt * ni_over_nt * (1.0 - perp * perp);
    if descriminant > 0.0 {
        Some(ni_over_nt * (direction - perp * normal) - normal * f32::sqrt(descriminant))
    } else {
        None
    }
}

pub fn schlick(cosine: f32, index_of_reflection: f32) -> f32 {
    let r0 = (1.0 - index_of_reflection) / (1.0 + index_of_reflection);
    let r0_sq = r0 * r0;
    let r1 = 1.0 - cosine;
    let r1_5 = r1 * r1 * r1 * r1 * r1;
    r0_sq + (1.0 - r0_sq) * r1_5
}

impl Material {
    pub fn scatter<T: Rng>(
        &self,
        ray: &Ray,
        hit: &RayHit,
        attenuation: &mut Color,
        rng: &mut T,
        between: &Uniform<f32>,
        atlas: &TextureAtlas,
    ) -> Option<Ray> {
        use Material::*;
        match self {
            &Lambertian { albedo } => {
                *attenuation = atlas.evaluate(albedo, hit.u, hit.v, hit.location);
                let target = hit.location + hit.normal + Vec3::from(UnitBall.sample(rng));
                Some(Ray::look_at(hit.location, target))
            }
            &Metal { albedo, fuzziness } => {
                *attenuation = atlas.evaluate(albedo, hit.u, hit.v, hit.location);
                let reflected_direction = reflect(ray.direction, hit.normal);
                let freedom = f32::min(1.0f32, fuzziness);
                let offset: Vec3 = reflected_direction + Vec3::from(UnitBall.sample(rng)) * freedom;
                let next_ray = Ray::look_at(hit.location, hit.location + offset);
                // Only cast a new ray if we notice we are sending it to the outside
                // If it would bounce back in, we just stop... Is this desired behaviour?
                // I do not know.
                if next_ray.direction.dot(&hit.normal) >= 0f32 {
                    Some(next_ray)
                } else {
                    None
                }
            }
            &Dielectric {
                index_of_refraction,
            } => {
                let ray_norm = ray.direction.dot(&hit.normal);
                let (outward_normal, ni_over_nt, cosine) = if ray_norm > 0.0 {
                    (
                        -1.0 * hit.normal,
                        index_of_refraction,
                        index_of_refraction * ray_norm,
                    )
                } else {
                    (hit.normal, 1.0 / index_of_refraction, -ray_norm)
                };
                *attenuation = Color::new(1.0, 1.0, 1.0);
                // If we can calculate a refraction offset
                if let Some(offset) = refract(ray.direction, outward_normal, ni_over_nt) {
                    // And we roll that we should refract (based on our angle i.e. schlick)
                    if rng.sample(between) > schlick(cosine, index_of_refraction) {
                        // Then return that we should follow a ray off towards there
                        return Some(Ray::look_at(hit.location, hit.location + offset));
                    }
                }
                // In all other cases, we will reflect
                let offset = reflect(ray.direction, hit.normal);
                Some(Ray::look_at(hit.location, hit.location + offset))
            }
            &Emissive { .. } => None,
            &Isotropic { albedo } => {
                *attenuation = atlas.evaluate(albedo, hit.u, hit.v, hit.location);
                let out_dir = UnitSphere.sample(rng).into();
                Some(Ray::new(hit.location, out_dir))
            }
            &Projective => {
                *attenuation = Color::new(1.0, 1.0, 1.0);
                Some(Ray::new(
                    -hit.location,
                    Quaternion::axis_angle(hit.normal, std::f32::consts::PI) * ray.direction,
                ))
            }
            &DirectionCollapse => {
                *attenuation = Color::new(1.0, 1.0, 1.0);
                Some(Ray::new(
                    -hit.location.length() * ray.direction,
                    ray.direction,
                ))
            }
        }
    }

    pub fn emit<T: Rng>(
        &self,
        ray_hit: &RayHit,
        _rng: &mut T,
        _between: &Uniform<f32>,
        atlas: &TextureAtlas,
    ) -> Color {
        use Material::*;
        match self {
            &Emissive { texture, amplify } => {
                amplify * atlas.evaluate(texture, ray_hit.u, ray_hit.v, ray_hit.location)
            }
            _ => Color::zero(),
        }
    }
}
