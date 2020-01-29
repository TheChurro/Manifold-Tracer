use super::open_cl_kernels::TRACE_KERNEL;
use crate::geometry::three_sphere::*;
use std::fmt::{Display, Error as FormatError, Formatter};

use ocl::error::{Error as OclError, Result as OclResult};
use ocl::{
    enums::{ImageChannelDataType, ImageChannelOrder, MemObjectType},
    flags::{MEM_HOST_READ_ONLY, MEM_WRITE_ONLY},
    Image, SpatialDims,
};
use ocl::{prm::Float4, Buffer};
use ocl::{Context, Device, Kernel, Program, Queue};

use std::ffi::CString;

use image::RgbaImage;

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
    pub output_image: Image<u8>,
    pub origins_buffer: Buffer<Float4>,
    pub tangents_buffer: Buffer<Float4>,
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
            num_rays: None,
            dims: None,
            edge_ab_normals: Vec::new(),
            edge_bc_normals: Vec::new(),
            edge_ca_normals: Vec::new(),
            normals: Vec::new(),
            ball_centers: Vec::new(),
            ball_radii: Vec::new(),
        })
    }

    pub fn trace(&self, rays: &Vec<(Point, Point)>, img: &mut RgbaImage) -> OclResult<()> {
        let mut origins: Vec<Float4> = Vec::new();
        let mut tangents: Vec<Float4> = Vec::new();
        for ray in rays {
            origins.push(ray.0.into());
            tangents.push(ray.1.into());
        }
        self.origins_buffer.write(&origins).enq()?;
        self.tangents_buffer.write(&tangents).enq()?;
        unsafe { self.kernel.enq()? };
        self.output_image.read(img).enq()?;
        Ok(())
    }
}

pub struct TraceKernelBuilder {
    pub kernel_program: Program,
    pub num_rays: Option<u32>,
    pub dims: Option<SpatialDims>,
    pub edge_ab_normals: Vec<Float4>,
    pub edge_bc_normals: Vec<Float4>,
    pub edge_ca_normals: Vec<Float4>,
    pub normals: Vec<Float4>,
    pub ball_centers: Vec<Float4>,
    pub ball_radii: Vec<f32>,
}

#[derive(Clone, Copy, Debug)]
pub enum TraceKernelBufferID {
    OriginsBuffer,
    TangentsBuffer,
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
    match Buffer::builder()
        .len(vals.len())
        .queue(queue.clone())
        .copy_host_slice(vals)
        .build()
    {
        Ok(buffer) => Ok(buffer),
        Err(e) => Err(TraceKernelBuildError::BufferBuildError(buffer_id, e)),
    }
}

impl TraceKernelBuilder {
    pub fn with_rays(mut self, num_rays: u32) -> Self {
        self.num_rays = Some(num_rays);
        self
    }

    pub fn with_dims(mut self, width: u32, height: u32) -> Self {
        self.dims = Some((width, height).into());
        self
    }

    pub fn load_triangle(&mut self, tri: &Triangle) {
        self.edge_ab_normals.push(tri.edge_normals[0].into());
        self.edge_bc_normals.push(tri.edge_normals[1].into());
        self.edge_ca_normals.push(tri.edge_normals[2].into());
        self.normals.push(tri.triangle_normal.into());
    }

    pub fn load_ball(&mut self, center: &Point, radius: f32) {
        self.ball_centers.push(center.into());
        self.ball_radii.push(radius);
    }

    pub fn build(&self, queue: &Queue) -> Result<TraceKernel, TraceKernelBuildError> {
        use TraceKernelBufferID::*;
        use TraceKernelBuildError::*;
        let num_rays = match self.num_rays {
            Some(num_rays) => num_rays,
            None => return Err(NoNumberOfRaysSet),
        };
        let dims = match self.dims {
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
        let origins_buffer = match Buffer::builder().len(num_rays).queue(queue.clone()).build() {
            Ok(buffer) => buffer,
            Err(e) => return Err(BufferBuildError(OriginsBuffer, e)),
        };
        let tangents_buffer = match Buffer::builder().len(num_rays).queue(queue.clone()).build() {
            Ok(buffer) => buffer,
            Err(e) => return Err(BufferBuildError(TangentsBuffer, e)),
        };
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
            .global_work_size(dims)
            .arg(&output_image)
            .arg(&origins_buffer)
            .arg(&tangents_buffer)
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
                output_image: output_image,
                origins_buffer: origins_buffer,
                tangents_buffer: tangents_buffer,
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
