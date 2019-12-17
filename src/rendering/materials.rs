use rand::distributions::Uniform;
use rand::Rng;

use crate::math::colors::Color;
use crate::math::ray::{Ray, RayHit};
use crate::math::vectors::Vec3;

#[derive(Copy, Clone)]
pub enum Material {
    Lambertian { albedo: Color },
    Metal { albedo: Color, fuzziness: f32 },
}

fn point_in_sphere<T: Rng>(rng: &mut T, between: &Uniform<f32>) -> Vec3 {
    loop {
        let random_vec = Vec3 {
            x: rng.sample(between),
            y: rng.sample(between),
            z: rng.sample(between),
        };
        let random_vec_adjusted = random_vec * 2.0f32 - Vec3::new(1f32, 1f32, 1f32);
        if random_vec_adjusted.length_sq() <= 1.0f32 {
            return random_vec_adjusted;
        }
    }
}

pub fn reflect(target: Vec3, normal: Vec3) -> Vec3 {
    target - 2f32 * target.dot(&normal) * normal
}

impl Material {
    pub fn scatter<T: Rng>(
        &self,
        ray: &Ray,
        hit: RayHit,
        attenuation: &mut Color,
        rng: &mut T,
        between: &Uniform<f32>,
    ) -> Option<Ray> {
        use Material::*;
        match self {
            &Lambertian { albedo } => {
                *attenuation = albedo;
                let target = hit.location + hit.normal + point_in_sphere(rng, between);
                Some(Ray::look_at(hit.location, target))
            }
            &Metal { albedo, fuzziness } => {
                *attenuation = albedo;
                let reflected_direction = reflect(ray.direction, hit.normal);
                let freedom = f32::min(1.0f32, fuzziness);
                let offset = reflected_direction + point_in_sphere(rng, between) * freedom;
                let next_ray = Ray::look_at(hit.location, hit.location + offset);
                // Only cast a new ray if we notice we are on the outside
                if next_ray.direction.dot(&hit.normal) > 0f32 {
                    Some(next_ray)
                } else {
                    None
                }
            }
        }
    }
}
