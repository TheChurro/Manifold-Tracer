use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};
use image::{Rgb, Pixel};

#[derive(Clone, Copy, Debug)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

op_impl!(Color, Add, add, r, g, b);
op_assign_impl!(Color, AddAssign, add_assign, r, g, b);
op_impl!(Color, Sub, sub, r, g, b);
op_assign_impl!(Color, SubAssign, sub_assign, r, g, b);
op_impl!(Color, Mul, mul, r, g, b);
op_assign_impl!(Color, MulAssign, mul_assign, r, g, b);
op_impl!(Color, Div, div, r, g, b);
op_assign_impl!(Color, DivAssign, div_assign, r, g, b);

op_scalar_impl!(Color, f32, Add, add, r, g, b);
op_scalar_assign_impl!(Color, f32, AddAssign, add_assign, r, g, b);
op_scalar_impl!(Color, f32, Sub, sub, r, g, b);
op_scalar_assign_impl!(Color, f32, SubAssign, sub_assign, r, g, b);
op_scalar_impl!(Color, f32, Mul, mul, r, g, b);
op_scalar_assign_impl!(Color, f32, MulAssign, mul_assign, r, g, b);
op_scalar_impl!(Color, f32, Div, div, r, g, b);
op_scalar_assign_impl!(Color, f32, DivAssign, div_assign, r, g, b);

impl From<Rgb<u8>> for Color
{
    fn from(val : Rgb<u8>) -> Color
    {
        Color {
            r : f32::from(val.channels()[0]) / 255f32,
            g : f32::from(val.channels()[1]) / 255f32,
            b : f32::from(val.channels()[2]) / 255f32
        }
    }
}

impl From<Color> for Rgb<u8>
{
    fn from(color : Color) -> Rgb<u8>
    {
        Rgb([
            f32::floor(color.r * 255f32) as u8,
            f32::floor(color.g * 255f32) as u8,
            f32::floor(color.b * 255f32) as u8,
        ])
    }
}
