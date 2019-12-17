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

const MIN_TIME: f32 = 0.001;
const MAX_TIME: f32 = 1000.0;
const MAX_ITERATIONS: u32 = 50;

fn color<T: Rng>(mut ray: Ray, scene: &Scene, rng: &mut T, between: &Uniform<f32>) -> Color {
    let mut color_absorbed = Color::new(1.0, 1.0, 1.0);
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
            let t = 0.5 * (ray.direction.y + 1.0);
            return color_absorbed
                * Color::lerp(Color::new(1.0, 1.0, 1.0), Color::new(0.5, 0.7, 1.0), t);
        }
    }
    // If we go for so long that we say we hit nothing, color the point black.
    Color::zero()
}

#[allow(dead_code)]
fn create_scene() -> Scene {
    let mut scene = Scene::new();
    scene.put(
        Collider::new(SphereGeometry {
            center: Vec3::new(0.0, 0.0, -1.0),
            radius: 0.5,
        }),
        Material::Lambertian {
            albedo: Color::new(0.8, 0.3, 0.3),
        },
    );
    scene.put(
        Collider::new(SphereGeometry {
            center: Vec3::new(0.0, -100.5, -1.0),
            radius: 100.0,
        }),
        Material::Lambertian {
            albedo: Color::new(0.5, 1.0, 0.5),
        },
    );
    scene.put(
        Collider::new(SphereGeometry {
            center: Vec3::new(1.0, 0.0, -1.0),
            radius: 0.5,
        }),
        Material::Metal {
            albedo: Color::new(0.8, 0.6, 0.2),
            fuzziness: 0.0,
        },
    );
    scene.put(
        Collider::new(SphereGeometry {
            center: Vec3::new(-1.0, -0.0, -1.0),
            radius: 0.5,
        }),
        Material::Dielectric {
            index_of_refraction: 1.5,
        },
    );
    scene.put(
        Collider::new(SphereGeometry {
            center: Vec3::new(-1.0, -0.0, -1.0),
            radius: -0.45,
        }),
        Material::Dielectric {
            index_of_refraction: 1.5,
        },
    );
    scene.put(
        Collider::new(SphereGeometry {
            center: Vec3::new(-1.0, -0.0, -1.0),
            radius: 0.25,
        }),
        Material::Metal {
            albedo: Color::new(0.5, 0.5, 1.0),
            fuzziness: 0.0,
        },
    );
    scene
}

#[allow(dead_code)]
fn test_scene_two() -> Scene {
    let mut scene = Scene::new();
    let r = std::f32::consts::FRAC_PI_4.cos();
    scene.put(
        Collider::new(SphereGeometry {
            center: Vec3::new(-r, 0.0, -1.0),
            radius: r,
        }),
        Material::Lambertian {
            albedo: Color::new(0.0, 0.0, 1.0),
        },
    );
    scene.put(
        Collider::new(SphereGeometry {
            center: Vec3::new(r, 0.0, -1.0),
            radius: r,
        }),
        Material::Lambertian {
            albedo: Color::new(1.0, 0.0, 0.0),
        },
    );
    scene
}

fn sample_color<T: Rng>(rng: &mut T) -> Color {
    let mat_range = Uniform::new(0.0, 1.0);
    Color::new(
        rng.sample(mat_range) * rng.sample(mat_range),
        rng.sample(mat_range) * rng.sample(mat_range),
        rng.sample(mat_range) * rng.sample(mat_range)
    )
}

#[allow(dead_code)]
fn random_scene() -> Scene {
    let mut scene = Scene::new();
    let mut rng = thread_rng();
    let mat_range = Uniform::new(0.0, 1.0);
    let dielectric_range = Uniform::new(0.8, 2.0);
    let radius_range = Uniform::new(0.1, 0.3);
    let pos_range = Uniform::new(-0.45, 0.45);
    // Put on a floor
    scene.put(
        Collider::new(SphereGeometry {
            center: Vec3::new(0.0, -1000.0, -1.0),
            radius: 1000.0,
        }),
        Material::Lambertian {
            albedo: Color::new(0.5, 0.5, 0.5),
        },
    );
    for x in -11..12 {
        for z in -11..12 {
            let choosen_material = rng.sample(mat_range);
            let radius = rng.sample(radius_range);
            let pos = Vec3::new(
                x as f32 + rng.sample(pos_range),
                radius,
                z as f32 + rng.sample(pos_range)
            );
            if (pos - Vec3::new(4.0, 0.2, 0.0)).length() > 0.9
            {
                if choosen_material < 0.8 {
                    scene.put(
                        Collider::new(SphereGeometry {
                            center: pos,
                            radius: radius,
                        }),
                        Material::Lambertian {
                            albedo: sample_color(&mut rng),
                        },
                    )
                } else if choosen_material < 0.95 {
                    scene.put(
                        Collider::new(SphereGeometry {
                            center: pos,
                            radius: radius,
                        }),
                        Material::Metal {
                            albedo: sample_color(&mut rng),
                            fuzziness: rng.sample(mat_range) * rng.sample(mat_range)
                        },
                    )
                } else {
                    scene.put(
                        Collider::new(SphereGeometry {
                            center: pos,
                            radius: radius,
                        }),
                        Material::Dielectric {
                            index_of_refraction: rng.sample(dielectric_range)
                        },
                    )
                }
            }
        }
    }

    scene.put(
        Collider::new(SphereGeometry {
            center: Vec3::new(0.0, 1.0, 0.0),
            radius: 1.0,
        }),
        Material::Dielectric {
            index_of_refraction: 1.5
        },
    );
    scene.put(
        Collider::new(SphereGeometry {
            center: Vec3::new(4.0, 1.0, 0.0),
            radius: 1.0,
        }),
        Material::Metal {
            albedo: Color::new(0.7, 0.6, 0.5),
            fuzziness: 0.0
        },
    );
    scene.put(
        Collider::new(SphereGeometry {
            center: Vec3::new(-4.0, 1.0, 0.0),
            radius: 1.0,
        }),
        Material::Lambertian {
            albedo: Color::new(0.4, 0.2, 0.1),
        },
    );

    scene
}

fn main() {
    let width = 1200;
    let height = 800;
    let aspect = width as f32 / height as f32;
    let num_samples = 50;
    let mut tmp_image = RgbImage::new(width, height);
    let location = Vec3::new(13.0, 2.0, 3.0);
    let look_at = Vec3::new(0.0, 0.0, 0.0);
    let camera = Camera::new(
        location,
        look_at,
        Vec3::up(),
        50.0,
        aspect,
        0.1,
        10.0
    );
    let scene = random_scene();
    let mut rng = thread_rng();
    let between = Uniform::new(0.0, 1.0);

    let time = std::time::Instant::now();

    for x in 0..width {
        for y in 0..height {
            let mut color_accumulator = Color::new(0.0, 0.0, 0.0);
            for _ in 0..num_samples {
                let u = (x as f32 + between.sample(&mut rng)) / (width as f32);
                let v = (y as f32 + between.sample(&mut rng)) / (height as f32);
                let ray = camera.world_ray(u, v);
                color_accumulator += color(ray, &scene, &mut rng, &between);
            }
            color_accumulator /= num_samples as f32;
            let out_color = color_accumulator.gamma2_correct();
            tmp_image.put_pixel(x, y, Rgb::from(out_color));
        }
    }
    println!("Time to render: {}", time.elapsed().as_millis());

    tmp_image
        .save("output/test.png")
        .expect("Failed to save image.");
}
