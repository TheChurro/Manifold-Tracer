extern crate image;
extern crate indicatif;
extern crate rand;

pub mod math;
pub mod rendering;

use image::RgbImage;
use indicatif::{ProgressBar, ProgressStyle};
use rand::distributions::{Distribution, Uniform};
use rand::rngs::SmallRng;
use rand::{thread_rng, Rng, SeedableRng};

use math::colliders::Collider;
use math::colors::Color;
use math::geometry::sphere::SphereGeometry;
use math::geometry::rect::RectGeometry;
use math::ray::Ray;
use math::vectors::Vec3;

use rendering::bvh::BVHNode;
use rendering::camera::Camera;
use rendering::materials::Material;
use rendering::scene::Scene;
use rendering::textures::{SampleMode, Texture, TextureIndex};

const MIN_TIME: f32 = 0.001;
const MAX_TIME: f32 = 1000.0;
const MAX_ITERATIONS: u32 = 50;

fn color<T: Rng>(mut ray: Ray, scene: &Scene, rng: &mut T, between: &Uniform<f32>) -> Color {
    let mut confirmed_color = Color::zero();
    let mut color_absorbed = Color::new(1.0, 1.0, 1.0);
    for _ in 0..MAX_ITERATIONS {
        if let Some((hit, material)) = scene.cast(&ray, MIN_TIME, MAX_TIME) {
            let mut attenuation = Color::zero();
            confirmed_color +=
                color_absorbed * material.emit(&hit, rng, between, &scene.texture_atlas);
            if let Some(new_ray) = material.scatter(
                &ray,
                &hit,
                &mut attenuation,
                rng,
                between,
                &scene.texture_atlas,
            ) {
                color_absorbed *= attenuation;
                ray = new_ray.cast_at(ray.cast_time);
            } else {
                return confirmed_color;
            }
        // ray = ;
        } else {
            // Sky box coloring. Where does the ray hit out at infinity.
            let t = 0.5 * (ray.direction.y + 1.0);
            return confirmed_color
                + color_absorbed
                    * Color::lerp(Color::new(1.0, 1.0, 1.0), Color::new(0.5, 0.7, 1.0), t);
        }
    }
    // If we go for so long that we say we hit nothing, color the point black.
    Color::zero()
}

#[allow(dead_code)]
fn create_scene() -> Scene {
    let mut scene = Scene::new();
    let checker_texture = Texture::CheckerVolume(
        TextureIndex::Constant(Color::new(0.2, 0.3, 0.1)),
        TextureIndex::Constant(Color::new(0.9, 0.9, 0.9)),
        0.5,
    );
    let checker_texture = scene.add_texture(checker_texture);
    scene.put(
        SphereGeometry {
            center: Vec3::new(0.0, 0.0, -1.0),
            radius: 0.5,
        }.into(),
        Material::Lambertian {
            albedo: TextureIndex::Constant(Color::new(0.8, 0.3, 0.3)),
        },
    );
    scene.put(
        SphereGeometry {
            center: Vec3::new(0.0, -100.5, -1.0),
            radius: 100.0,
        }.into(),
        Material::Lambertian {
            albedo: checker_texture,
        },
    );
    scene.put(
        SphereGeometry {
            center: Vec3::new(1.0, 0.0, -1.0),
            radius: 0.5,
        }.into(),
        Material::Metal {
            albedo: TextureIndex::Constant(Color::new(0.8, 0.6, 0.2)),
            fuzziness: 0.0,
        },
    );
    scene.put(
        SphereGeometry {
            center: Vec3::new(-1.0, -0.0, -1.0),
            radius: 0.5,
        }.into(),
        Material::Dielectric {
            index_of_refraction: 1.5,
        },
    );
    scene.put(
        SphereGeometry {
            center: Vec3::new(-1.0, -0.0, -1.0),
            radius: -0.45,
        }.into(),
        Material::Dielectric {
            index_of_refraction: 1.5,
        },
    );
    scene.put(
        SphereGeometry {
            center: Vec3::new(-1.0, -0.0, -1.0),
            radius: 0.25,
        }.into(),
        Material::Metal {
            albedo: TextureIndex::Constant(Color::new(0.5, 0.5, 1.0)),
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
        SphereGeometry {
            center: Vec3::new(-r, 0.0, -1.0),
            radius: r,
        }.into(),
        Material::Lambertian {
            albedo: TextureIndex::Constant(Color::new(0.0, 0.0, 1.0)),
        },
    );
    scene.put(
        SphereGeometry {
            center: Vec3::new(r, 0.0, -1.0),
            radius: r,
        }.into(),
        Material::Lambertian {
            albedo: TextureIndex::Constant(Color::new(1.0, 0.0, 0.0)),
        },
    );
    scene
}

fn sample_color<T: Rng>(rng: &mut T) -> Color {
    let mat_range = Uniform::new(0.0, 1.0);
    Color::new(
        rng.sample(mat_range) * rng.sample(mat_range),
        rng.sample(mat_range) * rng.sample(mat_range),
        rng.sample(mat_range) * rng.sample(mat_range),
    )
}

const SEED: [u8; 16] = [
    0x10u8, 0x72u8, 0x1Fu8, 0xEAu8, 0x7Au8, 0x40u8, 0xF2u8, 0x7Eu8, 0xB2u8, 0xF5u8, 0xCDu8, 0xC6u8,
    0x39u8, 0x66u8, 0xA3u8, 0x38u8,
];

#[allow(dead_code)]
fn random_scene() -> Scene {
    let mut scene = Scene::new();

    let mut rng = SmallRng::from_seed(SEED);
    let mat_range = Uniform::new(0.0, 1.0);
    let dielectric_range = Uniform::new(0.8, 2.0);
    let radius_range = Uniform::new(0.1, 0.3);
    let pos_range = Uniform::new(-0.45, 0.45);
    let speed_range = Uniform::new(-1.0, 1.0);
    let checker_texture = Texture::CheckerVolume(
        TextureIndex::Constant(Color::new(0.2, 0.3, 0.1)),
        TextureIndex::Constant(Color::new(0.9, 0.9, 0.9)),
        1.0,
    );
    let checker_texture = scene.add_texture(checker_texture);
    scene.put(
        SphereGeometry {
            center: Vec3::new(0.0, -1000.0, -1.0),
            radius: 1000.0,
        }.into(),
        Material::Lambertian {
            albedo: checker_texture,
        },
    );
    for x in -11..12 {
        for z in -11..12 {
            let choosen_material = rng.sample(mat_range);
            let radius = rng.sample(radius_range);
            let pos = Vec3::new(
                x as f32 + rng.sample(pos_range),
                radius,
                z as f32 + rng.sample(pos_range),
            );
            if (pos - Vec3::new(4.0, 0.2, 0.0)).length() > 0.9 {
                let mut collider: Collider = SphereGeometry {
                    center: pos,
                    radius: radius,
                }.into();
                let y_speed = rng.sample(speed_range);
                if y_speed > 0.0 {
                    collider = collider.with_velocity(Vec3::new(
                        rng.sample(speed_range),
                        y_speed,
                        rng.sample(speed_range),
                    ));
                }
                if choosen_material < 0.8 {
                    scene.put(
                        collider,
                        Material::Lambertian {
                            albedo: TextureIndex::Constant(sample_color(&mut rng)),
                        },
                    )
                } else if choosen_material < 0.95 {
                    scene.put(
                        collider,
                        Material::Metal {
                            albedo: TextureIndex::Constant(sample_color(&mut rng)),
                            fuzziness: rng.sample(mat_range) * rng.sample(mat_range),
                        },
                    )
                } else {
                    scene.put(
                        collider,
                        Material::Dielectric {
                            index_of_refraction: rng.sample(dielectric_range),
                        },
                    )
                }
            }
        }
    }

    scene.put(
        SphereGeometry {
            center: Vec3::new(0.0, 1.0, 0.0),
            radius: 1.0,
        }.into(),
        Material::Dielectric {
            index_of_refraction: 1.5,
        },
    );
    scene.put(
        SphereGeometry {
            center: Vec3::new(4.0, 1.0, 0.0),
            radius: 1.0,
        }.into(),
        Material::Metal {
            albedo: TextureIndex::Constant(Color::new(0.7, 0.6, 0.5)),
            fuzziness: 0.0,
        },
    );
    scene.put(
        SphereGeometry {
            center: Vec3::new(-4.0, 1.0, 0.0),
            radius: 1.0,
        }.into(),
        Material::Lambertian {
            albedo: TextureIndex::Constant(Color::new(0.4, 0.2, 0.1)),
        },
    );

    scene
}

#[allow(dead_code)]
fn test_textures_scene(aspect: f32) -> (Scene, Camera) {
    let mut scene = Scene::new();
    let camera_pos = Vec3::new(0.0, 1.0, -2.0);
    let scene_center = Vec3::new(0.0, 1.0, 2.0);

    let green_tex = scene.add_texture(Texture::Constant(Color::new(0.2, 0.3, 0.1)));
    let white_tex = scene.add_texture(Texture::Constant(Color::new(0.9, 0.9, 0.9)));
    let red_tex = scene.add_texture(Texture::Constant(Color::new(1.0, 0.3, 0.1)));
    let blue_tex = scene.add_texture(Texture::Constant(Color::new(0.1, 0.3, 1.0)));
    let checker_vol_tex = scene.add_texture(Texture::CheckerVolume(green_tex, white_tex, 0.5));
    let checker_surf_tex = scene.add_texture(Texture::CheckerSurface(red_tex, blue_tex, 20));
    let marble_tex = scene.add_texture(Texture::Noise(
        2.0,
        7,
        0.5,
        Box::new(|x: Vec3, y: Vec3| {
            (0.5 * (1.0 + (4.0 * x.z + 10.0 * y.length_sq()).sin())).into()
        }),
    ));
    let earth_tex = scene.add_texture(Texture::Image(
        image::open("resources/earthmap.jpg")
            .expect("Failed to load earth image!")
            .to_rgb(),
        SampleMode::Wrap,
    ));

    let test_sphere = SphereGeometry::new(Vec3::new(0.0, 1.0, 2.0), 1.0);
    let ground_sphere = SphereGeometry::new(-100.0 * Vec3::up(), 100.0);
    let mirror_quad = RectGeometry::new(4.0 * Vec3::forward(), 2.0, 1.0);

    let ground_material = Material::Lambertian {
        albedo: checker_vol_tex,
    };
    let marble_material = Material::Lambertian { albedo: marble_tex };
    let checker_material = Material::Emissive {
        texture: checker_surf_tex,
        amplify: 1.2,
    };
    let earth_material = Material::Metal { albedo: earth_tex, fuzziness: 1.0 };
    let mirror_material = Material::Metal { albedo: white_tex, fuzziness: 0.001 };

    scene.put(
        test_sphere.offset(2.0 * Vec3::right()).into(),
        marble_material,
    );
    scene.put(
        test_sphere.offset(Vec3::up()).into(),
        checker_material,
    );
    scene.put(
        test_sphere.offset(-2.0 * Vec3::right()).into(),
        earth_material,
    );
    scene.put(mirror_quad.into(), mirror_material);
    scene.put(ground_sphere.into(), ground_material);
    (
        scene,
        Camera::new(
            camera_pos,
            scene_center,
            Vec3::up(),
            80.0,
            aspect,
            0.1,
            (camera_pos - scene_center).length(),
        ),
    )
}

#[allow(dead_code)]
fn test_perlin_two_spheres(aspect: f32) -> (Scene, Camera) {
    let mut scene = Scene::new();
    let camera_pos = Vec3::new(13.0, 2.0, 3.0);
    let scene_center = Vec3::new(0.0, 0.0, 0.0);

    let marble_tex = scene.add_texture(Texture::Noise(
        2.0,
        7,
        0.5,
        Box::new(|x: Vec3, y: Vec3| {
            (0.5 * (1.0 + (4.0 * x.z + 10.0 * y.length_sq()).sin())).into()
        }),
    ));
    let turb_tex = scene.add_texture(Texture::Turbulence(2.0, 7, 0.5));
    let tex_sphere = SphereGeometry::new(2.0 * Vec3::up(), 2.0);
    let ground_sphere = SphereGeometry::new(-1000.0 * Vec3::up(), 1000.0);
    let perlin_material = Material::Metal {
        albedo: marble_tex,
        fuzziness: 0.75,
    };
    let ground_material = Material::Lambertian { albedo: turb_tex };

    scene.put(tex_sphere.into(), perlin_material);
    scene.put(ground_sphere.into(), ground_material);
    (
        scene,
        Camera::new(
            camera_pos,
            scene_center,
            Vec3::up(),
            90.0,
            aspect,
            0.0,
            10.0,
        ),
    )
}

fn main() {
    let width = 1200 / 5;
    let height = 800 / 5;
    let aspect = width as f32 / height as f32;
    let num_samples = 50;
    let delta_time = 1.0 / 30.0;

    let mut tmp_image = RgbImage::new(width, height);

    let (mut scene, camera) = test_textures_scene(aspect);
    scene.compute_hierarchy(0.0, delta_time);
    if let &Some(ref hierarchy) = &scene.hierarchy {
        let mut total_volume = 0.0;
        let mut num_volumes = 0.0;
        for node in &hierarchy.hierarchy_heap {
            if let BVHNode::Split(geom) = node {
                num_volumes += 1.0;
                total_volume += geom.volume();
            }
        }
        println!("Average Bounding Volume: {}", total_volume / num_volumes);
    }

    let mut rng = thread_rng();
    let between = Uniform::new(0.0, 1.0);

    let time = std::time::Instant::now();

    let progress_bar = ProgressBar::new((width * height * num_samples) as u64);
    let sty = ProgressStyle::default_bar()
        .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
        .progress_chars("#>-");
    progress_bar.set_style(sty);

    for x in 0..width {
        for y in 0..height {
            let mut color_accumulator = Color::new(0.0, 0.0, 0.0);
            progress_bar.set_message(&format!("row: {} | col: {}", x, y));
            for _ in 0..num_samples {
                let u = (x as f32 + between.sample(&mut rng)) / (width as f32);
                let v = (y as f32 + between.sample(&mut rng)) / (height as f32);
                let ray = camera
                    .world_ray(u, v)
                    .cast_at(delta_time * rng.sample(between));
                color_accumulator += color(ray, &scene, &mut rng, &between);
                progress_bar.inc(1);
            }
            color_accumulator /= num_samples as f32;
            let out_color = color_accumulator.gamma2_correct();
            tmp_image.put_pixel(x, y, out_color.into());
        }
    }
    progress_bar.finish_with_message("done!");
    println!("Time to render: {}", time.elapsed().as_millis());

    tmp_image
        .save("output/test.png")
        .expect("Failed to save image.");
}
