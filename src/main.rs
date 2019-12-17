extern crate image;

pub mod math;
pub mod rendering;

use image::{Rgb, RgbImage};

use math::colors::Color;
use math::ray::Ray;
use math::vectors::Vec3;
use rendering::camera::Camera;

fn color(ray : &Ray) -> Color {
    let t = 0.5f32 * (ray.direction.y + 1.0f32);
    return Color::lerp(Color::new(1.0, 1.0, 1.0), Color::new(0.5, 0.7, 1.0), t);
}

fn main() {
    let width = 200;
    let height = 100;
    let mut tmp_image = RgbImage::new(width, height);
    let camera = Camera {
        location: Vec3::zero(),
        extents: Vec3::new(2.0, 1.0, 1.0),
    };

    for x in 1..width {
        for y in 1..height {
            let u = (x as f32) / (width as f32);
            let v = (y as f32) / (height as f32);
            let ray = Ray::look_at(camera.location, camera.world_point(u, v));
            tmp_image.put_pixel(x, y, Rgb::from(color(&ray)));
        }
    }

    tmp_image
        .save("output/test.png")
        .expect("Failed to save image.");
}
