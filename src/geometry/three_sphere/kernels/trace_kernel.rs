use super::open_cl_kernels::TRACE_KERNEL;
use super::wavefront::InOutBufferSet;
use crate::geometry::three_sphere::*;

use std::fmt::{Display, Error as FormatError, Formatter};

use ocl::error::{Error as OclError, Result as OclResult};
use ocl::{prm::Float4, Buffer};
use ocl::{Context, Device, Kernel, Program, Queue};

use std::ffi::CString;

impl From<Point> for Float4 {
    fn from(p: Point) -> Float4 {
        Float4::new(p.w, p.i, p.j, p.k)
    }
}

impl From<&Point> for Float4 {
    fn from(p: &Point) -> Float4 {
        Float4::new(p.w, p.i, p.j, p.k)
    }
}

pub struct TraceKernel {
    pub kernel: Kernel,
    pub edge_ab_normals_buffer: Buffer<Float4>,
    pub edge_bc_normals_buffer: Buffer<Float4>,
    pub edge_ca_normals_buffer: Buffer<Float4>,
    pub normals_buffer: Buffer<Float4>,
    pub ball_centers_buffer: Buffer<Float4>,
    pub ball_radii_buffer: Buffer<f32>,
}

impl TraceKernel {
    pub fn builder(context: &Context, device: &Device) -> OclResult<TraceKernelBuilder> {
        let program_source = CString::new(TRACE_KERNEL)
            .unwrap_or_else(|e| panic!("Failed to turn trace kernel into CString: {}", e));
        let program = Program::with_source(
            context,
            &[program_source],
            Some(&[device.clone()]),
            &CString::default(),
        )?;
        Ok(TraceKernelBuilder {
            kernel_program: program,
            edge_ab_normals: Vec::new(),
            edge_bc_normals: Vec::new(),
            edge_ca_normals: Vec::new(),
            normals: Vec::new(),
            ball_centers: Vec::new(),
            ball_radii: Vec::new(),
        })
    }

    pub fn run(&mut self, num_rays: u32) -> Result<(), ocl::Error> {
        // Update how many rays we are sending into the scene
        self.kernel.set_arg(4, &num_rays)?;
        unsafe { self.kernel.enq()? };
        Ok(())
    }
}

pub struct TraceKernelBuilder {
    pub kernel_program: Program,
    pub edge_ab_normals: Vec<Float4>,
    pub edge_bc_normals: Vec<Float4>,
    pub edge_ca_normals: Vec<Float4>,
    pub normals: Vec<Float4>,
    pub ball_centers: Vec<Float4>,
    pub ball_radii: Vec<f32>,
}

#[derive(Clone, Copy, Debug)]
pub enum TraceKernelBufferID {
    EdgeABNormalsBuffer,
    EdgeBCNormalsBuffer,
    EdgeCANormalsBuffer,
    NormalsBuffer,
    BallCentersBuffer,
    BallRadiiBuffer,
}

#[derive(Debug)]
pub enum TraceKernelBuildError {
    NoNumberOfRaysSet,
    NoDimsSet,
    BufferBuildError(TraceKernelBufferID, OclError),
    ImageBuildError(OclError),
    KernelBuildError(OclError),
}

impl Display for TraceKernelBuildError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FormatError> {
        use TraceKernelBuildError::*;
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
    buffer_id: TraceKernelBufferID,
    queue: &Queue,
) -> Result<Buffer<T>, TraceKernelBuildError> {
    let mut builder = Buffer::builder()
        .len(vals.len().max(1))
        .queue(queue.clone());
    if vals.len() > 0 {
        builder = builder.copy_host_slice(vals);
    }
    match builder.build() {
        Ok(buffer) => Ok(buffer),
        Err(e) => Err(TraceKernelBuildError::BufferBuildError(buffer_id, e)),
    }
}

impl TraceKernelBuilder {
    pub fn load_triangle(&mut self, tri: &Triangle) {
        self.edge_ab_normals.push(tri.edge_normals[0].into());
        self.edge_bc_normals.push(tri.edge_normals[1].into());
        self.edge_ca_normals.push(tri.edge_normals[2].into());
        self.normals.push(tri.triangle_normal.into());
    }

    pub fn load_ball(&mut self, ball: &Ball) {
        self.ball_centers.push(ball.center.into());
        self.ball_radii.push(ball.radius);
    }

    pub fn build(
        &self,
        num_rays: u32,
        queue: &Queue,
        in_out_buffers: &InOutBufferSet,
    ) -> Result<TraceKernel, TraceKernelBuildError> {
        use TraceKernelBufferID::*;
        use TraceKernelBuildError::*;
        let edge_ab_normals_buffer =
            build_and_load_buffer(&self.edge_ab_normals, EdgeABNormalsBuffer, queue)?;
        let edge_bc_normals_buffer =
            build_and_load_buffer(&self.edge_bc_normals, EdgeBCNormalsBuffer, queue)?;
        let edge_ca_normals_buffer =
            build_and_load_buffer(&self.edge_ca_normals, EdgeCANormalsBuffer, queue)?;
        let normals_buffer = build_and_load_buffer(&self.normals, NormalsBuffer, queue)?;
        let ball_centers_buffer =
            build_and_load_buffer(&self.ball_centers, BallCentersBuffer, queue)?;
        let ball_radii_buffer = build_and_load_buffer(&self.ball_radii, BallRadiiBuffer, queue)?;
        match Kernel::builder()
            .program(&self.kernel_program)
            .name("trace")
            .global_work_size(num_rays)
            .arg(&in_out_buffers.trace_ray_origin)
            .arg(&in_out_buffers.trace_ray_tangent)
            .arg(&in_out_buffers.trace_ray_color)
            .arg(&in_out_buffers.trace_ray_info)
            .arg(&0)
            .arg(&in_out_buffers.shade_ray_origin)
            .arg(&in_out_buffers.shade_ray_tangent)
            .arg(&in_out_buffers.shade_ray_color)
            .arg(&in_out_buffers.shade_ray_info)
            .arg(&in_out_buffers.hit_normal)
            .arg(&edge_ab_normals_buffer)
            .arg(&edge_bc_normals_buffer)
            .arg(&edge_ca_normals_buffer)
            .arg(&normals_buffer)
            .arg(&self.normals.len())
            .arg(&ball_centers_buffer)
            .arg(&ball_radii_buffer)
            .arg(&self.ball_radii.len())
            .queue(queue.clone())
            .build()
        {
            Ok(kernel) => Ok(TraceKernel {
                kernel: kernel,
                edge_ab_normals_buffer: edge_ab_normals_buffer,
                edge_bc_normals_buffer: edge_bc_normals_buffer,
                edge_ca_normals_buffer: edge_ca_normals_buffer,
                normals_buffer: normals_buffer,
                ball_centers_buffer: ball_centers_buffer,
                ball_radii_buffer: ball_radii_buffer,
            }),
            Err(e) => Err(KernelBuildError(e)),
        }
    }
}
