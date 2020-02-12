use super::open_cl_kernels::SHADE_KERNEL;
use super::wavefront::InOutBufferSet;
use std::fmt::{Display, Error as FormatError, Formatter};

use ocl::error::{Error as OclError, Result as OclResult};
use ocl::{prm::Float4, Buffer};
use ocl::{Context, Device, Kernel, Program, Queue};

use std::ffi::CString;

pub struct ShadeKernel {
    pub kernel: Kernel,
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
            triangle_colors: Vec::new(),
            triangle_mat_types: Vec::new(),
            ball_colors: Vec::new(),
            ball_mat_types: Vec::new(),
        })
    }

    pub fn shade(&mut self, num_rays: u32) -> OclResult<()> {
        self.kernel.set_arg(6, &num_rays)?;
        unsafe { self.kernel.enq()? };
        Ok(())
    }
}

use crate::geometry::three_sphere::object::MaterialType;
pub struct ShadeKernelBuilder {
    pub kernel_program: Program,
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

fn build_and_load_buffer<T: ocl::OclPrm + Default>(
    vals: &Vec<T>,
    buffer_id: ShadeKernelBufferID,
    queue: &Queue,
) -> Result<Buffer<T>, ShadeKernelBuildError> {
    let mut builder = Buffer::builder()
        .len(vals.len().max(1))
        .queue(queue.clone());
    if vals.len() > 0 {
        builder = builder.copy_host_slice(vals);
    }
    match builder.build() {
        Ok(buffer) => Ok(buffer),
        Err(e) => Err(ShadeKernelBuildError::BufferBuildError(buffer_id, e)),
    }
}

impl ShadeKernelBuilder {
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
        num_rays: u32,
        in_out_buffers: &InOutBufferSet,
        queue: &Queue,
    ) -> Result<ShadeKernel, ShadeKernelBuildError> {
        use ShadeKernelBufferID::*;
        use ShadeKernelBuildError::*;
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
            .global_work_size(num_rays)
            .arg(&in_out_buffers.sample_color_out)
            .arg(&in_out_buffers.shade_ray_origin)
            .arg(&in_out_buffers.shade_ray_tangent)
            .arg(&in_out_buffers.shade_ray_color)
            .arg(&in_out_buffers.shade_ray_info)
            .arg(&in_out_buffers.hit_normal)
            .arg(&0)
            .arg(&in_out_buffers.trace_ray_origin)
            .arg(&in_out_buffers.trace_ray_tangent)
            .arg(&in_out_buffers.trace_ray_color)
            .arg(&in_out_buffers.trace_ray_info)
            .arg(&in_out_buffers.shade_rays_produced)
            .arg(&triangle_colors_buffer)
            .arg(&triangle_mat_types_buffer)
            .arg(&ball_colors_buffer)
            .arg(&ball_mat_types_buffer)
            .queue(queue.clone())
            .build()
        {
            Ok(kernel) => Ok(ShadeKernel {
                kernel: kernel,
                triangle_colors_buffer: triangle_colors_buffer,
                triangle_mat_types_buffer: triangle_mat_types_buffer,
                ball_colors_buffer: ball_colors_buffer,
                ball_mat_types_buffer: ball_mat_types_buffer,
            }),
            Err(e) => Err(KernelBuildError(e)),
        }
    }
}
