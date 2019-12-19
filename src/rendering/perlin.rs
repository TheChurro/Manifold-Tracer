use rand::distributions::Uniform;
use rand::{thread_rng, Rng};

use crate::math::vectors::Vec3;

const PERLIN_POW: usize = 8;
const PERLIN_SIZE: usize = 1 << PERLIN_POW;
const PERLIN_MASK: usize = PERLIN_SIZE - 1;

pub struct Perlin {
    pub noise: [Vec3; PERLIN_SIZE],
    pub sigma_x: [usize; PERLIN_SIZE],
    pub sigma_y: [usize; PERLIN_SIZE],
    pub sigma_z: [usize; PERLIN_SIZE],
}

fn linear(array: &mut [usize]) -> &mut [usize] {
    for i in 0..array.len() {
        array[i] = i;
    }
    array
}

fn permute<T: Rng>(array: &mut [usize], rng: &mut T) {
    let distro = Uniform::new(0, array.len());
    for i in 0..array.len() {
        let swap_index = rng.sample(distro);
        array.swap(i, swap_index);
    }
}

fn perlin_interp(array: &[[[Vec3; 2]; 2]; 2], u: f32, v: f32, w: f32) -> f32 {
    let mut accum = 0.0;
    let uu = u * u * (3.0 - 2.0 * u);
    let vv = v * v * (3.0 - 2.0 * v);
    let ww = w * w * (3.0 - 2.0 * w);
    for i in 0..2 {
        for j in 0..2 {
            for k in 0..2 {
                let weight = Vec3::new(u - (i as f32), v - (j as f32), w - (k as f32));
                accum += ((i as f32) * uu + (1.0 - (i as f32)) * (1.0 - uu))
                    * ((j as f32) * vv + (1.0 - (j as f32)) * (1.0 - vv))
                    * ((k as f32) * ww + (1.0 - (k as f32)) * (1.0 - ww))
                    * array[i][j][k].dot(&weight);
            }
        }
    }
    accum
}

fn perlin_interp_vec(array: &[[[Vec3; 2]; 2]; 2], u: f32, v: f32, w: f32) -> Vec3 {
    let mut accum = Vec3::zero();
    let uu = u * u * (3.0 - 2.0 * u);
    let vv = v * v * (3.0 - 2.0 * v);
    let ww = w * w * (3.0 - 2.0 * w);
    for i in 0..2 {
        for j in 0..2 {
            for k in 0..2 {
                accum += ((i as f32) * uu + (1.0 - (i as f32)) * (1.0 - uu))
                    * ((j as f32) * vv + (1.0 - (j as f32)) * (1.0 - vv))
                    * ((k as f32) * ww + (1.0 - (k as f32)) * (1.0 - ww))
                    * array[i][j][k];
            }
        }
    }
    accum
}

fn random_unit_vector<T: Rng>(rng: &mut T, between: &Uniform<f32>) -> Vec3 {
    let u = rng.sample(between);
    let v = rng.sample(between);
    let theta = u * 2.0 * std::f32::consts::PI;
    let phi = (2.0 * v - 1.0).acos();
    Vec3::new(theta.cos() * phi.cos(), phi.sin(), theta.sin() * phi.cos())
}

impl Perlin {
    pub fn new() -> Perlin {
        let between = Uniform::new(0.0, 1.0);
        let mut rng = thread_rng();

        let mut noise = [Vec3::zero(); PERLIN_SIZE];
        for i in 0..PERLIN_SIZE {
            noise[i] = random_unit_vector(&mut rng, &between);
        }
        let mut sigma_x = [0; PERLIN_SIZE];
        let mut sigma_y = [0; PERLIN_SIZE];
        let mut sigma_z = [0; PERLIN_SIZE];
        permute(linear(&mut sigma_x), &mut rng);
        permute(linear(&mut sigma_y), &mut rng);
        permute(linear(&mut sigma_z), &mut rng);
        Perlin {
            noise: noise,
            sigma_x: sigma_x,
            sigma_y: sigma_y,
            sigma_z: sigma_z,
        }
    }

    pub fn noise(&self, point: Vec3) -> f32 {
        let u = point.x - point.x.floor();
        let v = point.y - point.y.floor();
        let w = point.z - point.z.floor();
        let i = f32::floor(point.x) as usize;
        let j = f32::floor(point.y) as usize;
        let k = f32::floor(point.z) as usize;
        let mut c = [[[Vec3::zero(); 2]; 2]; 2];
        for di in 0..2 {
            for dj in 0..2 {
                for dk in 0..2 {
                    c[di][dj][dk] = self.noise[self.sigma_x[(i + di) & PERLIN_MASK]
                        ^ self.sigma_y[(j + dj) & PERLIN_MASK]
                        ^ self.sigma_z[(k + dk) & PERLIN_MASK]]
                }
            }
        }
        perlin_interp(&c, u, v, w)
    }

    pub fn noise_vec(&self, point: Vec3) -> Vec3 {
        let u = point.x - point.x.floor();
        let v = point.y - point.y.floor();
        let w = point.z - point.z.floor();
        let i = f32::floor(point.x) as usize;
        let j = f32::floor(point.y) as usize;
        let k = f32::floor(point.z) as usize;
        let mut c = [[[Vec3::zero(); 2]; 2]; 2];
        for di in 0..2 {
            for dj in 0..2 {
                for dk in 0..2 {
                    c[di][dj][dk] = self.noise[self.sigma_x[(i + di) & PERLIN_MASK]
                        ^ self.sigma_y[(j + dj) & PERLIN_MASK]
                        ^ self.sigma_z[(k + dk) & PERLIN_MASK]]
                }
            }
        }
        perlin_interp_vec(&c, u, v, w)
    }

    pub fn turbulence(&self, mut point: Vec3, depth: u32, ratio: f32) -> f32 {
        let mut accum = 0.0;
        let mut weight = 1.0;
        for _ in 0..depth {
            accum += weight * self.noise(point);
            weight *= ratio;
            point /= ratio;
        }
        accum
    }

    pub fn turbulence_vec(&self, mut point: Vec3, depth: u32, ratio: f32) -> Vec3 {
        let mut accum = Vec3::zero();
        let mut weight = 1.0;
        let mut accum_weight = 0.0;
        for _ in 0..depth {
            accum += weight * self.noise_vec(point);
            accum_weight += weight;
            weight *= ratio;
            point /= ratio;
        }
        accum / accum_weight
    }
}
