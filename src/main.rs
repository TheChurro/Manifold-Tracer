extern crate image;

pub mod math;

use image::{Rgb, RgbImage};

fn main() {
    let width = 200;
    let height = 100;
    let mut tmp_image = RgbImage::new(width, height);

    for x in 1..width {
        for y in 1..height {
            let red = ((x * 255) / width) as u8;
            let green = ((y * 255) / height) as u8;
            let blue = ((2 * 255) / 10) as u8;
            tmp_image.put_pixel(x, y, Rgb([red, green, blue]));
        }
    }

    tmp_image
        .save("output/test.png")
        .expect("Failed to save image.");
}
