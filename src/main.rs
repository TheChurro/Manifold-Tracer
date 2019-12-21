extern crate image;
extern crate indicatif;
extern crate rand;
extern crate rand_distr;
extern crate structopt;
#[macro_use]
extern crate clap;

pub mod math;
pub mod rendering;

use image::RgbImage;
use indicatif::{ProgressBar, ProgressStyle};
use rand::distributions::{Distribution, Uniform};
use rand::rngs::SmallRng;
use rand::{thread_rng, Rng, SeedableRng};
use structopt::StructOpt;

use math::colliders::Collider;
use math::colors::Color;
use math::geometry::rect::RectGeometry;
use math::geometry::sphere::SphereGeometry;
use math::quaternion::Quaternion;
use math::ray::Ray;
use math::vectors::Vec3;

use rendering::bvh::BVHNode;
use rendering::camera::Camera;
use rendering::materials::Material;
use rendering::scene::Scene;
use rendering::textures::{SampleMode, Texture, TextureIndex};

const MIN_TIME: f32 = 0.001;
const MAX_TIME: f32 = std::f32::MAX;
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
        } else {
            return confirmed_color;
        }
    }
    // If we go for so long that we say we hit nothing, color the point black.
    confirmed_color
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
        }
        .into(),
        Material::Lambertian {
            albedo: TextureIndex::Constant(Color::new(0.8, 0.3, 0.3)),
        },
    );
    scene.put(
        SphereGeometry {
            center: Vec3::new(0.0, -100.5, -1.0),
            radius: 100.0,
        }
        .into(),
        Material::Lambertian {
            albedo: checker_texture,
        },
    );
    scene.put(
        SphereGeometry {
            center: Vec3::new(1.0, 0.0, -1.0),
            radius: 0.5,
        }
        .into(),
        Material::Metal {
            albedo: TextureIndex::Constant(Color::new(0.8, 0.6, 0.2)),
            fuzziness: 0.0,
        },
    );
    scene.put(
        SphereGeometry {
            center: Vec3::new(-1.0, -0.0, -1.0),
            radius: 0.5,
        }
        .into(),
        Material::Dielectric {
            index_of_refraction: 1.5,
        },
    );
    scene.put(
        SphereGeometry {
            center: Vec3::new(-1.0, -0.0, -1.0),
            radius: -0.45,
        }
        .into(),
        Material::Dielectric {
            index_of_refraction: 1.5,
        },
    );
    scene.put(
        SphereGeometry {
            center: Vec3::new(-1.0, -0.0, -1.0),
            radius: 0.25,
        }
        .into(),
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
        }
        .into(),
        Material::Lambertian {
            albedo: TextureIndex::Constant(Color::new(0.0, 0.0, 1.0)),
        },
    );
    scene.put(
        SphereGeometry {
            center: Vec3::new(r, 0.0, -1.0),
            radius: r,
        }
        .into(),
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
        }
        .into(),
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
                }
                .into();
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
        }
        .into(),
        Material::Dielectric {
            index_of_refraction: 1.5,
        },
    );
    scene.put(
        SphereGeometry {
            center: Vec3::new(4.0, 1.0, 0.0),
            radius: 1.0,
        }
        .into(),
        Material::Metal {
            albedo: TextureIndex::Constant(Color::new(0.7, 0.6, 0.5)),
            fuzziness: 0.0,
        },
    );
    scene.put(
        SphereGeometry {
            center: Vec3::new(-4.0, 1.0, 0.0),
            radius: 1.0,
        }
        .into(),
        Material::Lambertian {
            albedo: TextureIndex::Constant(Color::new(0.4, 0.2, 0.1)),
        },
    );

    scene
}

fn create_box(extents: Vec3) -> Collider {
    let mut faces = Vec::new();
    faces.push(Collider::from(RectGeometry::new(
        -extents.z * Vec3::forward(),
        2.0 * extents.x,
        2.0 * extents.y,
    )));
    faces.push(
        Collider::from(RectGeometry::new(
            -extents.z * Vec3::forward(),
            2.0 * extents.x,
            2.0 * extents.y,
        ))
        .rotate(Quaternion::axis_angle(Vec3::up(), std::f32::consts::PI)),
    );
    faces.push(
        Collider::from(RectGeometry::new(
            -extents.x * Vec3::forward(),
            2.0 * extents.z,
            2.0 * extents.y,
        ))
        .rotate(Quaternion::axis_angle(
            Vec3::up(),
            std::f32::consts::FRAC_PI_2,
        )),
    );
    faces.push(
        Collider::from(RectGeometry::new(
            -extents.x * Vec3::forward(),
            2.0 * extents.z,
            2.0 * extents.y,
        ))
        .rotate(Quaternion::axis_angle(
            Vec3::up(),
            -std::f32::consts::FRAC_PI_2,
        )),
    );
    faces.push(
        Collider::from(RectGeometry::new(
            -extents.y * Vec3::forward(),
            2.0 * extents.x,
            2.0 * extents.z,
        ))
        .rotate(Quaternion::axis_angle(
            Vec3::right(),
            std::f32::consts::FRAC_PI_2,
        )),
    );
    faces.push(
        Collider::from(RectGeometry::new(
            -extents.y * Vec3::forward(),
            2.0 * extents.x,
            2.0 * extents.z,
        ))
        .rotate(Quaternion::axis_angle(
            Vec3::right(),
            -std::f32::consts::FRAC_PI_2,
        )),
    );
    Collider::Union(faces)
}

#[allow(dead_code)]
fn test_textures_scene(aspect: f32) -> (Scene, Camera) {
    let mut scene = Scene::new();
    let camera_pos = Vec3::new(0.0, 2.0, 0.0);
    let scene_center = Vec3::new(0.0, 1.0, 2.0);

    let green_tex = scene.add_texture(Texture::Constant(Color::new(0.2, 0.3, 0.1)));
    let white_tex = scene.add_texture(Texture::Constant(Color::new(0.9, 0.9, 0.9)));
    let red_tex = scene.add_texture(Texture::Constant(Color::new(1.0, 0.3, 0.1)));
    let checker_vol_tex = scene.add_texture(Texture::CheckerVolume(green_tex, white_tex, 0.5));
    let checker_surf_tex = scene.add_texture(Texture::CheckerSurface(red_tex, white_tex, 20));
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

    let test_sphere = SphereGeometry::new(Vec3::new(0.0, 0.0, 0.0), 1.0);
    let gas_sphere = SphereGeometry::new(Vec3::new(0.0, 0.0, 2.0), 5.0);
    let ground_sphere = SphereGeometry::new(-100.0 * Vec3::up(), 100.0);

    let ground_material = Material::Lambertian {
        albedo: checker_vol_tex,
    };
    let marble_material = Material::Lambertian { albedo: marble_tex };
    let checker_material = Material::Emissive {
        texture: checker_surf_tex,
        amplify: 2.0,
    };
    let earth_material = Material::Metal {
        albedo: earth_tex,
        fuzziness: 1.0,
    };
    let smoke_material = Material::Isotropic {
        albedo: white_tex,
    };

    scene.put(
        test_sphere.offset(Vec3::new(2.0, 1.0, 2.0)).into(),
        marble_material,
    );
    scene.put(
        test_sphere.offset(Vec3::new(0.0, 3.0, 2.0)).into(),
        checker_material,
    );
    scene.put(
        Collider::from(gas_sphere).to_volume(0.25),
        smoke_material,
    );
    scene.put(
        Collider::from(test_sphere)
            .translate(Vec3::new(-2.0, 1.0, 2.0)),
        earth_material,
    );
    //scene.put(mirror_sphere.into(), mirror_material);
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

#[allow(dead_code)]
fn cornell_box(aspect: f32) -> (Scene, Camera) {
    let mut scene = Scene::new();
    let camera_pos = Vec3::new(0.0, 275.0, -800.0);
    let scene_center = Vec3::new(0.0, 275.0, 0.0);

    let red_tex = scene.add_texture(Texture::Constant(Color::new(0.65, 0.05, 0.05)));
    let white_tex = scene.add_texture(Texture::Constant(Color::new(0.73, 0.73, 0.73)));
    let green_tex = scene.add_texture(Texture::Constant(Color::new(0.12, 0.45, 0.15)));
    let light_tex = scene.add_texture(Texture::Constant(Color::new(1.0, 1.0, 1.0)));

    let red_mat = Material::Lambertian { albedo: red_tex };
    let white_mat = Material::Lambertian { albedo: white_tex };
    let green_mat = Material::Lambertian { albedo: green_tex };
    let light_mat = Material::Emissive {
        texture: light_tex,
        amplify: 15.0,
    };

    let box_rect = RectGeometry::new(Vec3::zero(), 555.5, 555.5);
    let light_rect = RectGeometry::new(Vec3::zero(), 130.0, 105.0);

    scene.put(
        Collider::from(box_rect).translate(555.0 * 0.5 * Vec3::up() + 555.0 * Vec3::forward()),
        white_mat,
    );
    scene.put(
        Collider::from(box_rect)
            .rotate(Quaternion::axis_angle(
                Vec3::right(),
                std::f32::consts::FRAC_PI_2,
            ))
            .translate(555.0 * 0.5 * Vec3::forward()),
        white_mat,
    );
    scene.put(
        Collider::from(box_rect)
            .rotate(Quaternion::axis_angle(
                Vec3::right(),
                -std::f32::consts::FRAC_PI_2,
            ))
            .translate(555.0 * Vec3::up() + 555.0 * 0.5 * Vec3::forward()),
        white_mat,
    );
    scene.put(
        Collider::from(light_rect)
            .rotate(Quaternion::axis_angle(
                Vec3::right(),
                -std::f32::consts::FRAC_PI_2,
            ))
            .translate(554.5 * Vec3::up() + 555.0 * 0.5 * Vec3::forward()),
        light_mat,
    );
    scene.put(
        Collider::from(box_rect)
            .rotate(Quaternion::axis_angle(
                Vec3::up(),
                std::f32::consts::FRAC_PI_2,
            ))
            .translate(0.5 * Vec3::new(555.0, 555.0, 555.0)),
        green_mat,
    );
    scene.put(
        Collider::from(box_rect)
            .rotate(Quaternion::axis_angle(
                Vec3::up(),
                -std::f32::consts::FRAC_PI_2,
            ))
            .translate(0.5 * Vec3::new(-555.0, 555.0, 555.0)),
        red_mat,
    );
    scene.put(
        create_box(Vec3::new(82.5, 82.5, 82.5))
            .rotate(Quaternion::axis_angle(Vec3::up(), 18.0f32.to_radians()))
            .translate(Vec3::new(65.5, 82.5, 147.5)),
        white_mat,
    );
    scene.put(
        create_box(Vec3::new(82.5, 165.0, 82.5))
            .rotate(Quaternion::axis_angle(Vec3::up(), -15.0f32.to_radians()))
            .translate(Vec3::new(-72.5, 165.0, 377.5)),
        white_mat,
    );

    // Compute a vfov which covers the entire entry to the box.
    let half_height = 275.0f32 / 800.0;
    let half_theta = half_height.atan();
    let vfov = half_theta * 2.0 * 180.0 / std::f32::consts::PI;

    (
        scene,
        Camera::new(
            camera_pos,
            scene_center,
            Vec3::up(),
            vfov,
            aspect,
            0.0,
            800.0,
        ),
    )
}

#[allow(dead_code)]
fn cornell_box_with_haze(aspect: f32) -> (Scene, Camera) {
    let mut scene = Scene::new();
    let camera_pos = Vec3::new(0.0, 275.0, -800.0);
    let scene_center = Vec3::new(0.0, 275.0, 0.0);

    let red_tex = scene.add_texture(Texture::Constant(Color::new(0.65, 0.05, 0.05)));
    let white_tex = scene.add_texture(Texture::Constant(Color::new(0.73, 0.73, 0.73)));
    let green_tex = scene.add_texture(Texture::Constant(Color::new(0.12, 0.45, 0.15)));
    let light_tex = scene.add_texture(Texture::Constant(Color::new(1.0, 1.0, 1.0)));

    let red_mat = Material::Lambertian { albedo: red_tex };
    let white_mat = Material::Lambertian { albedo: white_tex };
    let green_mat = Material::Lambertian { albedo: green_tex };
    let light_mat = Material::Emissive {
        texture: light_tex,
        amplify: 7.0,
    };

    let pure_white_tex = scene.add_texture(Texture::Constant(Color::new(1.0, 1.0, 1.0)));
    let pure_black_tex = scene.add_texture(Texture::Constant(Color::zero()));
    let white_smoke_mat = Material::Isotropic { albedo: pure_white_tex };
    let black_smoke_mat = Material::Isotropic { albedo: pure_black_tex };

    let box_rect = RectGeometry::new(Vec3::zero(), 555.5, 555.5);
    let light_rect = RectGeometry::new(Vec3::zero(), 300.0, 300.0);

    scene.put(
        Collider::from(box_rect).translate(555.0 * 0.5 * Vec3::up() + 555.0 * Vec3::forward()),
        white_mat,
    );
    scene.put(
        Collider::from(box_rect)
            .rotate(Quaternion::axis_angle(
                Vec3::right(),
                std::f32::consts::FRAC_PI_2,
            ))
            .translate(555.0 * 0.5 * Vec3::forward()),
        white_mat,
    );
    scene.put(
        Collider::from(box_rect)
            .rotate(Quaternion::axis_angle(
                Vec3::right(),
                -std::f32::consts::FRAC_PI_2,
            ))
            .translate(555.0 * Vec3::up() + 555.0 * 0.5 * Vec3::forward()),
        white_mat,
    );
    scene.put(
        Collider::from(light_rect)
            .rotate(Quaternion::axis_angle(
                Vec3::right(),
                -std::f32::consts::FRAC_PI_2,
            ))
            .translate(554.5 * Vec3::up() + 555.0 * 0.5 * Vec3::forward()),
        light_mat,
    );
    scene.put(
        Collider::from(box_rect)
            .rotate(Quaternion::axis_angle(
                Vec3::up(),
                std::f32::consts::FRAC_PI_2,
            ))
            .translate(0.5 * Vec3::new(555.0, 555.0, 555.0)),
        green_mat,
    );
    scene.put(
        Collider::from(box_rect)
            .rotate(Quaternion::axis_angle(
                Vec3::up(),
                -std::f32::consts::FRAC_PI_2,
            ))
            .translate(0.5 * Vec3::new(-555.0, 555.0, 555.0)),
        red_mat,
    );
    scene.put(
        create_box(Vec3::new(82.5, 82.5, 82.5))
            .rotate(Quaternion::axis_angle(Vec3::up(), 18.0f32.to_radians()))
            .translate(Vec3::new(65.5, 82.5, 147.5))
            .to_volume(0.01),
        if false { white_mat } else { white_smoke_mat },
    );
    scene.put(
        create_box(Vec3::new(82.5, 165.0, 82.5))
            .rotate(Quaternion::axis_angle(Vec3::up(), -15.0f32.to_radians()))
            .translate(Vec3::new(-72.5, 165.0, 377.5))
            .to_volume(0.01),
        if false { white_mat } else { black_smoke_mat },
    );

    // Compute a vfov which covers the entire entry to the box.
    let half_height = 275.0f32 / 800.0;
    let half_theta = half_height.atan();
    let vfov = half_theta * 2.0 * 180.0 / std::f32::consts::PI;

    (
        scene,
        Camera::new(
            camera_pos,
            scene_center,
            Vec3::up(),
            vfov,
            aspect,
            0.0,
            800.0,
        ),
    )
}

#[allow(dead_code)]
fn cornell_projective_space(aspect: f32) -> (Scene, Camera) {
    let mut scene = Scene::new();
    let camera_pos = Vec3::new(0.0, 275.0, -500.0);
    let scene_center = Vec3::new(0.0, 275.0, 0.0);

    let red_tex = scene.add_texture(Texture::Constant(Color::new(0.65, 0.05, 0.05)));
    let white_tex = scene.add_texture(Texture::Constant(Color::new(0.73, 0.73, 0.73)));
    let green_tex = scene.add_texture(Texture::Constant(Color::new(0.12, 0.45, 0.15)));
    let light_tex = scene.add_texture(Texture::Constant(Color::new(1.0, 1.0, 1.0)));

    let red_mat = Material::Lambertian { albedo: red_tex };
    let white_mat = Material::Lambertian { albedo: white_tex };
    let green_mat = Material::Lambertian { albedo: green_tex };
    let light_mat = Material::Emissive {
        texture: light_tex,
        amplify: 15.0,
    };

    let box_rect = RectGeometry::new(Vec3::zero(), 200.0, 200.0);
    let light_rect = RectGeometry::new(Vec3::zero(), 100.0, 100.0);
    let projective_sphere = SphereGeometry::new(Vec3::zero(), -800.0);

    scene.put(projective_sphere.into(), green_mat);
    scene.put(
        Collider::from(box_rect).translate(555.0 * 0.5 * Vec3::up() + 555.0 * Vec3::forward()),
        white_mat,
    );
    scene.put(
        Collider::from(box_rect)
            .rotate(Quaternion::axis_angle(
                Vec3::right(),
                std::f32::consts::FRAC_PI_2,
            ))
            .translate(555.0 * 0.5 * Vec3::forward()),
        white_mat,
    );
    scene.put(
        Collider::from(box_rect)
            .rotate(Quaternion::axis_angle(
                Vec3::right(),
                -std::f32::consts::FRAC_PI_2,
            ))
            .translate(555.0 * Vec3::up() + 555.0 * 0.5 * Vec3::forward()),
        white_mat,
    );
    scene.put(
        Collider::from(light_rect)
            .rotate(Quaternion::axis_angle(
                Vec3::right(),
                -std::f32::consts::FRAC_PI_2,
            ))
            .translate(554.5 * Vec3::up() + 555.0 * 0.5 * Vec3::forward()),
        light_mat,
    );
    scene.put(
        Collider::from(box_rect)
            .rotate(Quaternion::axis_angle(
                Vec3::up(),
                std::f32::consts::FRAC_PI_2,
            ))
            .translate(0.5 * Vec3::new(555.0, 555.0, 555.0)),
        green_mat,
    );
    scene.put(
        Collider::from(box_rect)
            .rotate(Quaternion::axis_angle(
                Vec3::up(),
                -std::f32::consts::FRAC_PI_2,
            ))
            .translate(0.5 * Vec3::new(-555.0, 555.0, 555.0)),
        red_mat,
    );
    scene.put(
        create_box(Vec3::new(82.5, 82.5, 82.5))
            .rotate(Quaternion::axis_angle(Vec3::up(), 18.0f32.to_radians()))
            .translate(Vec3::new(65.5, 82.5, 147.5)),
        white_mat,
    );
    scene.put(
        create_box(Vec3::new(82.5, 165.0, 82.5))
            .rotate(Quaternion::axis_angle(Vec3::up(), -15.0f32.to_radians()))
            .translate(Vec3::new(-72.5, 165.0, 377.5)),
        white_mat,
    );

    // Compute a vfov which covers the entire entry to the box.
    let half_height = 275.0f32 / 500.0;
    let half_theta = half_height.atan();
    let vfov = half_theta * 2.0 * 180.0 / std::f32::consts::PI;

    (
        scene,
        Camera::new(
            camera_pos,
            scene_center,
            Vec3::up(),
            vfov,
            aspect,
            0.0,
            800.0,
        ),
    )
}

arg_enum!{
    #[derive(Debug)]
    enum ChoosenScene {
        MaterialTest,
        Cornell,
        CornellHaze,
        CornellProjectiveSpace
    }
}

#[derive(StructOpt, Debug)]
#[structopt(name = "manifold-tracer")]
struct Options {
    #[structopt(short, long, default_value = "200")]
    width: u32,
    #[structopt(short, long, default_value = "200")]
    height: u32,
    #[structopt(short, long, default_value = "10")]
    samples: u32,
    #[structopt(short, long, default_value = "output/test.png")]
    out_file: std::path::PathBuf,
    #[structopt(long, default_value = "MaterialTest")]
    scene: ChoosenScene,
    #[structopt(short, long)]
    update: Option<u32>
}

fn main() {
    let options = Options::from_args();

    let aspect = options.width as f32 / options.height as f32;
    let delta_time = 1.0 / 30.0;

    let mut tmp_image = RgbImage::new(options.width, options.height);

    let (mut scene, camera) = match options.scene {
        ChoosenScene::MaterialTest => test_textures_scene(aspect),
        ChoosenScene::Cornell => cornell_box(aspect),
        ChoosenScene::CornellHaze => cornell_box_with_haze(aspect),
        ChoosenScene::CornellProjectiveSpace => cornell_projective_space(aspect),
    };
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
    println!(
        "Camera: ({:?}, {:?}, {:?})",
        camera.horizontal, camera.vertical, camera.forward
    );

    let mut rng = thread_rng();
    let between = Uniform::new(0.0, 1.0);

    let time = std::time::Instant::now();

    let progress_bar = ProgressBar::new((options.width * options.height * options.samples) as u64);
    let sty = ProgressStyle::default_bar()
        .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
        .progress_chars("#>-");
    progress_bar.set_style(sty);

    // {
    //     let ray = camera.world_ray(180.0 / options.width as f32, 280.0 / options.height as f32);
    //     if let Some((hit, material)) = scene.cast(&ray, MIN_TIME, MAX_TIME) {
    //         println!("Ray: {:?}", ray);
    //         println!("Hit: {}", hit);
    //         println!("Material: {:?}", material);
    //     }
    //     if true {
    //         return;
    //     }
    // }

    for x in 0..options.width {
        for y in 0..options.height {
            let mut color_accumulator = Color::new(0.0, 0.0, 0.0);
            progress_bar.set_message(&format!("row: {} | col: {}", x, y));
            for _ in 0..options.samples {
                let u = (x as f32 + between.sample(&mut rng)) / (options.width as f32);
                let v = (y as f32 + between.sample(&mut rng)) / (options.height as f32);
                let ray = camera
                    .world_ray(u, v)
                    .cast_at(delta_time * rng.sample(between));
                color_accumulator += color(ray, &scene, &mut rng, &between);
                progress_bar.inc(1);
            }
            color_accumulator /= options.samples as f32;
            let out_color = color_accumulator.gamma2_correct();
            tmp_image.put_pixel(x, y, out_color.into());
        }
        if let Some(rows_to_update) = options.update {
            if x % rows_to_update == 0 {
                tmp_image
                    .save(options.out_file.clone())
                    .expect("Failed to save image.");
            }
        }
    }
    progress_bar.finish_with_message("done!");
    println!("Time to render: {}", time.elapsed().as_millis());

    tmp_image
        .save(options.out_file)
        .expect("Failed to save image.");
}
