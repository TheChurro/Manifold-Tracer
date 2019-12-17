extern crate image;
extern crate rand;

pub mod math;
pub mod rendering;

use image::{Rgb, RgbImage};
use rand::distributions::{Distribution, Uniform};
use rand::{thread_rng, Rng};

use math::colliders::{Collider, SphereGeometry};
use math::colors::Color;
use math::ray::Ray;
use math::vectors::Vec3;

use rendering::camera::Camera;
use rendering::materials::Material;
use rendering::scene::Scene;

const MIN_TIME: f32 = 0.001f32;
const MAX_TIME: f32 = 1000f32;
const MAX_ITERATIONS: u32 = 50;

fn color<T: Rng>(mut ray: Ray, scene: &Scene, rng: &mut T, between: &Uniform<f32>) -> Color {
    let mut color_absorbed = Color::new(1.0f32, 1.0f32, 1.0f32);
    for _ in 0..MAX_ITERATIONS {
        if let Some((hit, material)) = scene.cast(&ray, MIN_TIME, MAX_TIME) {
            let mut attenuation = Color::zero();
            if let Some(new_ray) = material.scatter(&ray, hit, &mut attenuation, rng, between) {
                color_absorbed *= attenuation;
                ray = new_ray;
            } else {
                return Color::zero();
            }
        // ray = ;
        } else {
            // Sky box coloring. Where does the ray hit out at infinity.
            let t = 0.5f32 * (ray.direction.y + 1.0f32);
            return color_absorbed
                * Color::lerp(Color::new(1.0, 1.0, 1.0), Color::new(0.5, 0.7, 1.0), t);
        }
    }
    // If we go for so long that we say we hit nothing, color the point black.
    Color::zero()
}

fn create_scene() -> Scene {
    let mut scene = Scene::new();
    scene.put(
        Collider::new(SphereGeometry {
            center: Vec3::new(0f32, 0f32, -1f32),
            radius: 0.5f32,
        }),
        Material::Lambertian {
            albedo: Color::new(0.8f32, 0.3f32, 0.3f32),
        },
    );
    scene.put(
        Collider::new(SphereGeometry {
            center: Vec3::new(0f32, -100.5f32, -1f32),
            radius: 100f32,
        }),
        Material::Lambertian {
            albedo: Color::new(0.5f32, 1.0f32, 0.5f32),
        },
    );
    scene.put(
        Collider::new(SphereGeometry {
            center: Vec3::new(1f32, 0f32, -1f32),
            radius: 0.5f32,
        }),
        Material::Metal {
            albedo: Color::new(0.8f32, 0.6f32, 0.2f32),
            fuzziness: 1.0f32
        },
    );
    scene.put(
        Collider::new(SphereGeometry {
            center: Vec3::new(-1f32, -0f32, -1f32),
            radius: 0.5f32,
        }),
        Material::Metal {
            albedo: Color::new(0.8f32, 0.8f32, 0.8f32),
            fuzziness: 0.3f32
        },
    );
    scene
}

fn main() {
    let width = 200;
    let height = 100;
    let num_samples = 100;
    let mut tmp_image = RgbImage::new(width, height);
    let camera = Camera {
        location: Vec3::zero(),
        extents: Vec3::new(2.0, 1.0, 1.0),
    };
    let scene = create_scene();
    let mut rng = thread_rng();
    let between = Uniform::new(0f32, 1f32);
    for x in 0..width {
        for y in 0..height {
            let mut color_accumulator = Color::new(0f32, 0f32, 0f32);
            for _ in 0..num_samples {
                let u = (x as f32 + between.sample(&mut rng)) / (width as f32);
                let v = (y as f32 + between.sample(&mut rng)) / (height as f32);
                let ray = Ray::look_at(camera.location, camera.world_point(u, v));
                color_accumulator += color(ray, &scene, &mut rng, &between);
            }
            color_accumulator /= num_samples as f32;
            let out_color = color_accumulator.gamma2_correct();
            tmp_image.put_pixel(x, y, Rgb::from(out_color));
        }
    }
    tmp_image
        .save("output/test.png")
        .expect("Failed to save image.");
}
