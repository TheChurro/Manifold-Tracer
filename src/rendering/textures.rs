use image::RgbImage;

use crate::math::colors::Color;
use crate::math::vectors::Vec3;

use crate::rendering::perlin::Perlin;

#[derive(Clone, Copy)]
pub enum TextureIndex {
    Constant(Color),
    Reference(usize),
}
pub struct TextureAtlas {
    pub textures: Vec<Texture>,
    pub perlin: Perlin,
}

impl TextureAtlas {
    pub fn new() -> TextureAtlas {
        TextureAtlas {
            textures: Vec::new(),
            perlin: Perlin::new(),
        }
    }

    pub fn add(&mut self, t: Texture) -> TextureIndex {
        match t {
            Texture::Constant(color) => TextureIndex::Constant(color),
            tex => {
                self.textures.push(tex);
                TextureIndex::Reference(self.textures.len() - 1)
            }
        }
    }

    pub fn evaluate(&self, mut tex_index: TextureIndex, u: f32, v: f32, point: Vec3) -> Color {
        loop {
            match tex_index {
                TextureIndex::Constant(color) => {
                    return color;
                }
                TextureIndex::Reference(index) => {
                    if index >= self.textures.len() {
                        return Color::zero();
                    }
                    use Texture::*;
                    match &self.textures[index] {
                        &Constant(color) => {
                            return color;
                        }
                        &CheckerVolume(left, right, rate) => {
                            let sin_coeff = std::f32::consts::PI / rate;
                            let sines = (point.x * sin_coeff).sin()
                                * (point.y * sin_coeff).sin()
                                * (point.z * sin_coeff).sin();
                            if sines < 0.0 {
                                tex_index = left;
                            } else {
                                tex_index = right;
                            }
                        }
                        &CheckerSurface(left, right, num_checkers) => {
                            let rate = 2.0 / num_checkers as f32;
                            let sin_coeff = std::f32::consts::PI / rate;
                            let sines = (u * sin_coeff).sin() * (v * sin_coeff).sin();
                            if sines < 0.0 {
                                tex_index = left;
                            } else {
                                tex_index = right;
                            }
                        }
                        Image(image, mode) => {
                            let x = f32::floor(image.width() as f32 * u);
                            let x = mode.adjust(x, image.width());
                            let y = f32::floor(image.height() as f32 * v);
                            let y = mode.adjust(y, image.height());
                            let color_ = image.get_pixel(x, y);
                            return color_.into();
                        }
                        Perlin(scale) => {
                            return (0.5 * (1.0 + self.perlin.noise(scale * point))).into();
                        }
                        Turbulence(scale, depth, ratio) => {
                            return self
                                .perlin
                                .turbulence(scale * point, *depth, *ratio)
                                .abs()
                                .into();
                        }
                        Noise(scale, depth, ratio, filter) => {
                            return filter(
                                scale * point,
                                self.perlin.turbulence_vec(scale * point, *depth, *ratio),
                            );
                        }
                    }
                }
            }
        }
    }
}

pub enum SampleMode {
    Clamp,
    Wrap,
}

impl SampleMode {
    pub fn adjust(&self, x: f32, width: u32) -> u32 {
        match self {
            &SampleMode::Clamp => {
                let x = x as i32;
                if x < 0 {
                    0
                } else if x as u32 >= width {
                    width - 1
                } else {
                    x as u32
                }
            }
            &SampleMode::Wrap => {
                let x_adj = if x < 0.0 {
                    width as f32 - (x.abs() % width as f32)
                } else {
                    x % width as f32
                };
                x_adj.floor() as u32
            }
        }
    }
}

pub enum Texture {
    Constant(Color),
    CheckerVolume(TextureIndex, TextureIndex, f32),
    CheckerSurface(TextureIndex, TextureIndex, u32),
    Image(RgbImage, SampleMode),
    Perlin(f32),
    Turbulence(f32, u32, f32),
    Noise(f32, u32, f32, Box<dyn Fn(Vec3, Vec3) -> Color>),
}
