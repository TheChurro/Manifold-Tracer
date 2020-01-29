use super::open_cl_kernels::SHADE_KERNEL;
use crate::geometry::three_sphere::*;
use std::fmt::{Display, Error as FormatError, Formatter};

use ocl::error::{Error as OclError, Result as OclResult};
use ocl::{
    enums::{ImageChannelDataType, ImageChannelOrder, MemObjectType},
    flags::{MEM_HOST_READ_ONLY, MEM_WRITE_ONLY},
    Image, SpatialDims,
};
use ocl::{
    prm::{Float4, Uint4},
    Buffer,
};
use ocl::{Context, Device, Kernel, Program, Queue};

use std::ffi::CString;

use image::RgbaImage;

pub struct ShadeKernel {
    pub kernel: Kernel,
    pub output_image: Image<u8>,
    pub origins_buffer: Buffer<Float4>,
    pub tangents_buffer: Buffer<Float4>,
    pub ray_colors_buffer: Buffer<Float4>,
    pub ray_hit_normals_buffer: Buffer<Float4>,
    pub ray_hit_infos_buffer: Buffer<Uint4>,
    pub triangle_colors_buffer: Buffer<Float4>,
    pub triangle_mat_types_buffer: Buffer<u32>,
    pub ball_colors_buffer: Buffer<Float4>,
    pub ball_mat_types_buffer: Buffer<u32>,
}

impl ShadeKernel {
    pub fn builder(context: &Context, device: &Device) -> OclResult<ShadeKernelBuilder> {
        let program_source = CString::new(SHADE_KERNEL)
            .unwrap_or_else(|e| panic!("Failed to turn trace kernel into CString: {}", e));
        let program = Program::with_source(
            context,
            &[program_source],
            Some(&[device.clone()]),
            &CString::default(),
        )?;
        Ok(ShadeKernelBuilder {
            kernel_program: program,
            image_dims: None,
            triangle_colors: Vec::new(),
            triangle_mat_types: Vec::new(),
            ball_colors: Vec::new(),
            ball_mat_types: Vec::new(),
        })
    }

    pub fn shade(&mut self, num_rays: u32, img: &mut RgbaImage) -> OclResult<u32> {
        self.kernel.set_default_global_work_size(num_rays.into());
        unsafe { self.kernel.enq()? };
        self.output_image.read(img).enq()?;
        Ok(0)
    }
}

pub enum MaterialType {
    Lambertian,
    Emissive,
}

pub struct ShadeKernelBuilder {
    pub kernel_program: Program,
    pub image_dims: Option<SpatialDims>,
    pub triangle_colors: Vec<Float4>,
    pub triangle_mat_types: Vec<u32>,
    pub ball_colors: Vec<Float4>,
    pub ball_mat_types: Vec<u32>,
}

#[derive(Clone, Copy, Debug)]
pub enum ShadeKernelBufferID {
    RayColorsBuffer,
    TriangleColorsBuffer,
    TriangleMatTypesBuffer,
    BallColorsBuffer,
    BallMatTypesBuffer,
}

#[derive(Debug)]
pub enum ShadeKernelBuildError {
    NoNumberOfRaysSet,
    NoDimsSet,
    BufferBuildError(ShadeKernelBufferID, OclError),
    ImageBuildError(OclError),
    KernelBuildError(OclError),
}

impl Display for ShadeKernelBuildError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FormatError> {
        use ShadeKernelBuildError::*;
        match self {
            NoNumberOfRaysSet => {
                write!(f, "Trace Kernel Builder requires a default number of rays.")
            }
            NoDimsSet => write!(
                f,
                "Trace Kernel Builder requires a default image dimension."
            ),
            BufferBuildError(id, error) => {
                write!(f, "Failed to build {:?} due to error: {}", id, error)
            }
            ImageBuildError(error) => write!(f, "Failed to build image due to error: {}", error),
            KernelBuildError(error) => write!(f, "Failed to build kernel due to error: {}", error),
        }
    }
}

fn build_and_load_buffer<T: ocl::OclPrm>(
    vals: &Vec<T>,
    buffer_id: ShadeKernelBufferID,
    queue: &Queue,
) -> Result<Buffer<T>, ShadeKernelBuildError> {
    match Buffer::builder()
        .len(vals.len())
        .queue(queue.clone())
        .copy_host_slice(vals)
        .build()
    {
        Ok(buffer) => Ok(buffer),
        Err(e) => Err(ShadeKernelBuildError::BufferBuildError(buffer_id, e)),
    }
}

impl ShadeKernelBuilder {
    pub fn with_dims(mut self, width: u32, height: u32) -> Self {
        self.image_dims = Some((width, height).into());
        self
    }

    pub fn add_triangle_material(&mut self, color: [f32; 3], mat_type: MaterialType) {
        self.triangle_colors
            .push(Float4::new(color[0], color[1], color[2], 1.0));
        self.triangle_mat_types.push(match mat_type {
            MaterialType::Lambertian => 0,
            MaterialType::Emissive => 1,
        });
    }

    pub fn add_ball_material(&mut self, color: [f32; 3], mat_type: MaterialType) {
        self.ball_colors
            .push(Float4::new(color[0], color[1], color[2], 1.0));
        self.ball_mat_types.push(match mat_type {
            MaterialType::Lambertian => 0,
            MaterialType::Emissive => 1,
        });
    }

    pub fn build(
        &self,
        max_rays: u32,
        ray_origins_buffer: &Buffer<Float4>,
        ray_tangents_buffer: &Buffer<Float4>,
        hit_normals_buffer: &Buffer<Float4>,
        hit_infos_buffer: &Buffer<Uint4>,
        queue: &Queue,
    ) -> Result<ShadeKernel, ShadeKernelBuildError> {
        use ShadeKernelBufferID::*;
        use ShadeKernelBuildError::*;
        let dims = match self.image_dims {
            Some(dims) => dims,
            None => return Err(NoDimsSet),
        };
        let output_image = match Image::<u8>::builder()
            .dims(dims)
            .channel_order(ImageChannelOrder::Rgba)
            .channel_data_type(ImageChannelDataType::UnormInt8)
            .image_type(MemObjectType::Image2d)
            .dims(&dims)
            .flags(MEM_WRITE_ONLY | MEM_HOST_READ_ONLY)
            .queue(queue.clone())
            .build()
        {
            Ok(img) => img,
            Err(e) => return Err(ImageBuildError(e)),
        };
        let mut initial_ray_colors = Vec::new();
        for _ in 0..max_rays {
            initial_ray_colors.push(Float4::new(1.0, 1.0, 1.0, 1.0));
        }
        let ray_colors_buffer = build_and_load_buffer(&initial_ray_colors, RayColorsBuffer, queue)?;
        let triangle_colors_buffer =
            build_and_load_buffer(&self.triangle_colors, TriangleColorsBuffer, queue)?;
        let triangle_mat_types_buffer =
            build_and_load_buffer(&self.triangle_mat_types, TriangleMatTypesBuffer, queue)?;
        let ball_colors_buffer = build_and_load_buffer(&self.ball_colors, BallColorsBuffer, queue)?;
        let ball_mat_types_buffer =
            build_and_load_buffer(&self.ball_mat_types, BallMatTypesBuffer, queue)?;
        match Kernel::builder()
            .program(&self.kernel_program)
            .name("shade")
            .global_work_size(max_rays)
            .arg(&output_image)
            .arg(ray_origins_buffer)
            .arg(ray_tangents_buffer)
            .arg(&ray_colors_buffer)
            .arg(hit_normals_buffer)
            .arg(hit_infos_buffer)
            .arg(&max_rays)
            .arg(&0)
            .arg(&triangle_colors_buffer)
            .arg(&triangle_mat_types_buffer)
            .arg(&ball_colors_buffer)
            .arg(&ball_mat_types_buffer)
            .queue(queue.clone())
            .build()
        {
            Ok(kernel) => Ok(ShadeKernel {
                kernel: kernel,
                output_image: output_image,
                origins_buffer: ray_origins_buffer.clone(),
                tangents_buffer: ray_tangents_buffer.clone(),
                ray_colors_buffer: ray_colors_buffer,
                ray_hit_normals_buffer: hit_normals_buffer.clone(),
                ray_hit_infos_buffer: hit_infos_buffer.clone(),
                triangle_colors_buffer: triangle_colors_buffer,
                triangle_mat_types_buffer: triangle_mat_types_buffer,
                ball_colors_buffer: ball_colors_buffer,
                ball_mat_types_buffer: ball_mat_types_buffer,
            }),
            Err(e) => Err(KernelBuildError(e)),
        }
    }
}
