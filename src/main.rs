extern crate image;
extern crate rand;

pub mod math;
pub mod rendering;

use image::{Rgb, RgbImage};
use rand::distributions::{Distribution, Uniform};
use rand::thread_rng;

use math::colliders::{Collider, SphereGeometry};
use math::colors::Color;
use math::ray::Ray;
use math::vectors::Vec3;

use rendering::camera::Camera;
use rendering::scene::Scene;

fn color(ray: &Ray, scene: &Scene) -> Color {
    if let Some(hit) = scene.cast(&ray) {
        0.5f32 * (1f32 + Color::new(hit.normal.x, hit.normal.y, hit.normal.z))
    } else {
        let t = 0.5f32 * (ray.direction.y + 1.0f32);
        Color::lerp(Color::new(1.0, 1.0, 1.0), Color::new(0.5, 0.7, 1.0), t)
    }
}

fn create_scene() -> Scene {
    let mut scene = Scene::new();
    scene.put(Collider::new(SphereGeometry{
        center: Vec3::new(0f32, 0f32, -1f32),
        radius: 0.5f32
    }));
    scene.put(Collider::new(SphereGeometry{
        center: Vec3::new(0f32, -100.5f32, -1f32),
        radius: 100f32
    }));
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
                color_accumulator += color(&ray, &scene);
            }
            color_accumulator /= num_samples as f32;

            tmp_image.put_pixel(x, y, Rgb::from(color_accumulator));
        }
    }

    tmp_image
        .save("output/test.png")
        .expect("Failed to save image.");
}
