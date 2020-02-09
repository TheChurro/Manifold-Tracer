use image::RgbaImage;
use manifold_tracer::geometry::three_sphere::{Direction, Orientation, Point};

use rand::{distributions::Distribution, distributions::Uniform, thread_rng};

pub struct CameraS3 {
    pub vfov: f32,
    pub aspect: f32,
    pub orientation: Orientation,
    pub theta: f32,
    pub azimuth: f32,
    pub samples: u32,
    pub image: RgbaImage,
}

impl CameraS3 {
    pub fn new(vfov: f32, width: u32, height: u32, num_samples: u32) -> CameraS3 {
        let img = RgbaImage::new(width, height);
        CameraS3 {
            vfov: vfov,
            aspect: width as f32 / height as f32,
            orientation: Orientation::identity(),
            theta: 0.0,
            azimuth: 0.0,
            samples: num_samples,
            image: img,
        }
    }
    pub fn get_interior_rotation(&self) -> Orientation {
        let azimuth = Orientation::rotate_towards(Direction::i(), 0.0, self.azimuth);
        let planar = Orientation::rotate_towards(Direction::j(), 0.0, self.theta);
        planar * azimuth
    }
    pub fn generate_rays_frustrum(&self) -> Vec<(Point, Point)> {
        let mut rays = Vec::new();
        let rot = self.orientation * self.get_interior_rotation();
        let right = &rot
            * Direction::i()
            * ((self.vfov / 2.0).sin() * self.aspect / self.image.width() as f32);
        let up = &rot * Direction::j() * ((self.vfov / 2.0).sin() / self.image.height() as f32);
        let forwards = &rot * Direction::k();
        let forwards_point = &rot * Point::k();
        let origin = rot * Point::one();
        let center_x = self.image.width() as f32 / 2.0;
        let center_y = self.image.height() as f32 / 2.0;
        let mut rng = thread_rng();
        let distro = Uniform::new_inclusive(0.0, 1.0);
        for x in 0..self.image.width() {
            let x_factor = x as f32 - center_x;
            for y in 0..self.image.height() {
                let y_factor = center_y - y as f32;
                for _ in 0..self.samples {
                    let x_jitter = distro.sample(&mut rng);
                    let y_jitter = distro.sample(&mut rng);
                    let tangent = Point::in_direction(
                        (x_factor + x_jitter) * right + (y_factor + y_jitter) * up + forwards,
                    )
                    .unwrap_or(forwards_point);
                    rays.push((origin, tangent));
                }
            }
        }
        rays
    }
    #[allow(dead_code)]
    pub fn generate_rays_rotationally(&self) -> Vec<(Point, Point)> {
        let mut rays = Vec::new();
        let rot = self.orientation * self.get_interior_rotation();
        let right = &rot * Direction::i();
        let up = &rot * Direction::j();
        let forwards = &rot * Direction::k();
        let forwards_point = &rot * Point::k();
        let origin = rot * Point::one();
        let center_x = self.image.width() as f32 / 2.0;
        let center_y = self.image.height() as f32 / 2.0;
        let xz_angle_factor = self.vfov * self.aspect / self.image.width() as f32;
        let yz_angle_factor = self.vfov / self.image.height() as f32;
        for x in 0..self.image.width() {
            let xz_angle = xz_angle_factor * (x as f32 - center_x);
            let xz_angle_cos = xz_angle.cos();
            let xz_angle_sin = xz_angle.sin();
            for y in 0..self.image.height() {
                let yz_angle = yz_angle_factor * (center_y - y as f32);
                let yz_angle_sin = yz_angle.sin();
                let yz_angle_cos = yz_angle.cos();
                let tangent = Point::in_direction(
                    xz_angle_sin * right
                        + yz_angle_sin * up
                        + xz_angle_cos * yz_angle_cos * forwards,
                )
                .unwrap_or(forwards_point);
                rays.push((origin, tangent));
            }
        }
        rays
    }
    pub fn rotate(&mut self, theta: f32, azimuth: f32) {
        use std::f32::consts::{FRAC_PI_2, PI};
        self.theta += theta;
        if self.theta < 0.0 {
            self.theta = 2.0 * PI + (self.theta % (2.0 * PI));
        } else {
            self.theta %= 2.0 * PI;
        }
        self.azimuth += azimuth;
        if self.azimuth < -FRAC_PI_2 {
            self.azimuth = -FRAC_PI_2;
        } else if self.azimuth > FRAC_PI_2 {
            self.azimuth = FRAC_PI_2;
        }
    }
    pub fn translate(&mut self, right: f32, up: f32, forwards: f32) {
        let target = self.get_interior_rotation() * (Direction::new(0.0, right, up, forwards));
        let move_length = target.magnitude();
        if let Some(p) = Point::in_direction(target) {
            self.orientation = self.orientation
                * Orientation::rotate_on_plane(&Point::one(), &p, move_length, 0.0);
        }
    }
}
