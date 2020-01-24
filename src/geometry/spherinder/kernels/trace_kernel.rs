use super::open_cl_kernels::TRACE_KERNEL;
use crate::geometry::sphere::OntoSphere;
use crate::geometry::spherinder::mesh::{Mesh, MeshTriangle};
use crate::geometry::spherinder::spaces::SpherinderPoint;
use crate::geometry::spherinder::triangle::Triangulable;
use std::fmt::{Display, Error as FormatError, Formatter};

use ocl::error::{Error as OclError, Result as OclResult};
use ocl::{
    enums::{ImageChannelDataType, ImageChannelOrder, MemObjectType},
    flags::{MEM_COPY_HOST_PTR, MEM_HOST_READ_ONLY, MEM_WRITE_ONLY},
    Image, SpatialDims,
};
use ocl::{
    prm::{Float4, Uint4},
    Buffer,
};
use ocl::{Context, Device, Kernel, Program, Queue};

use std::ffi::CString;

pub struct TraceKernel {
    pub kernel: Kernel,
    pub output_image: Image<u8>,
    pub origins_buffer: Buffer<Float4>,
    pub directions_buffer: Buffer<Float4>,
    pub verticies_buffer: Buffer<Float4>,
    pub triangles_buffer: Buffer<Uint4>,
    pub normals_buffer: Buffer<Float4>,
}

impl TraceKernel {
    pub fn builder(context: &Context, device: Device) -> OclResult<TraceKernelBuilder> {
        let program_source = CString::new(TRACE_KERNEL)
            .unwrap_or_else(|e| panic!("Failed to turn trace kernel into CString: {}", e));
        let program = Program::with_source(
            context,
            &[program_source],
            Some(&[device]),
            &CString::default(),
        )?;
        Ok(TraceKernelBuilder {
            kernel_program: program,
            num_rays: None,
            dims: None,
            verticies: Vec::new(),
            triangles: Vec::new(),
            normals: Vec::new(),
        })
    }
}

pub struct TraceKernelBuilder {
    pub kernel_program: Program,
    pub num_rays: Option<u32>,
    pub dims: Option<SpatialDims>,
    pub verticies: Vec<Float4>,
    pub triangles: Vec<Uint4>,
    pub normals: Vec<Float4>,
}

#[derive(Clone, Copy, Debug)]
pub enum TraceKernelBufferID {
    OriginsBuffer,
    DirectionsBuffer,
    VerticiesBuffer,
    TrianglesBuffer,
    NormalsBuffer,
}

#[derive(Debug)]
pub enum TraceKernelBuildError {
    NoNumberOfRaysSet,
    NoDimsSet,
    TriangleVertexOOB {
        vertex: usize,
        min: usize,
        max: usize,
        triangle: MeshTriangle,
    },
    BufferBuildError(TraceKernelBufferID, OclError),
    ImageBuildError(OclError),
    KernelBuildError(OclError),
}

impl Display for TraceKernelBuildError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FormatError> {
        use TraceKernelBuildError::*;
        match self {
            NoNumberOfRaysSet => write!(f, "Trace Kernel Builder requires a default number of rays."),
            NoDimsSet => write!(f, "Trace Kernel Builder requires a default image dimension."),
            TriangleVertexOOB {
                vertex,
                min,
                max,
                triangle: MeshTriangle(v0, v1, v2)
            } => write!(
                f,
                "Error in trying to add triangle ({}, {}, {})[({}, {}, {})].\n  Vertex {}[{}] out of bounds {} - {}",
                v0,
                v1,
                v2,
                v0 - min,
                v1 - min,
                v2 - min,
                vertex,
                vertex - min,
                min,
                max
            ),
            BufferBuildError(id, error) => write!(f, "Failed to build {:?} due to error: {}", id, error),
            ImageBuildError(error) => write!(f, "Failed to build image due to error: {}", error),
            KernelBuildError(error) => write!(f, "Failed to build kernel due to error: {}", error),
        }
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

    pub fn load_mesh<P: OntoSphere, T>(
        &mut self,
        mesh: &Mesh<P>,
    ) -> Result<(), TraceKernelBuildError>
    where
        SpherinderPoint<P>: Triangulable<P, T>,
    {
        let min_index = self.verticies.len();
        let mut new_verticies = Vec::new();
        for vertex in &mesh.geometry.points {
            let sphere_point = vertex.sphere.point();
            new_verticies.push(Float4::new(
                sphere_point.x,
                sphere_point.y,
                sphere_point.z,
                vertex.depth,
            ));
        }
        let max_index = new_verticies.len() + min_index;
        let mut new_triangles = Vec::new();
        let mut new_normals = Vec::new();
        for MeshTriangle(v0, v1, v2) in &mesh.tris {
            let v0_adj = v0 + min_index;
            let v1_adj = v1 + min_index;
            let v2_adj = v2 + min_index;
            let oob_vertex = if v0_adj >= max_index {
                Some(v0_adj)
            } else if v1_adj >= max_index {
                Some(v1_adj)
            } else if v2_adj >= max_index {
                Some(v2_adj)
            } else {
                None
            };
            if let Some(vertex) = oob_vertex {
                return Err(TraceKernelBuildError::TriangleVertexOOB {
                    vertex: vertex,
                    min: min_index,
                    max: max_index,
                    triangle: MeshTriangle(v0_adj, v1_adj, v2_adj),
                });
            }
            new_triangles.push(Uint4::new(v0_adj as u32, v1_adj as u32, v2_adj as u32, 0));
            let normal = mesh.geometry.points[*v0]
                .triangle_new(&mesh.geometry.points[*v1], &mesh.geometry.points[*v2])
                .normal;
            new_normals.push(normal);
        }
        Ok(())
    }

    pub fn build(&self, queue: &Queue) -> Result<TraceKernel, TraceKernelBuildError> {
        let num_rays = match self.num_rays {
            Some(num_rays) => num_rays,
            None => return Err(TraceKernelBuildError::NoNumberOfRaysSet),
        };
        let dims = match self.dims {
            Some(dims) => dims,
            None => return Err(TraceKernelBuildError::NoDimsSet),
        };
        let output_image = match Image::<u8>::builder()
            .dims(dims)
            .channel_order(ImageChannelOrder::Rgba)
            .channel_data_type(ImageChannelDataType::UnormInt8)
            .image_type(MemObjectType::Image2d)
            .dims(&dims)
            .flags(MEM_WRITE_ONLY | MEM_HOST_READ_ONLY | MEM_COPY_HOST_PTR)
            .build()
        {
            Ok(img) => img,
            Err(e) => return Err(TraceKernelBuildError::ImageBuildError(e)),
        };
        let origins_buffer = match Buffer::builder().len(num_rays).queue(queue.clone()).build() {
            Ok(buffer) => buffer,
            Err(e) => {
                return Err(TraceKernelBuildError::BufferBuildError(
                    TraceKernelBufferID::OriginsBuffer,
                    e,
                ))
            }
        };
        let directions_buffer = match Buffer::builder().len(num_rays).queue(queue.clone()).build() {
            Ok(buffer) => buffer,
            Err(e) => {
                return Err(TraceKernelBuildError::BufferBuildError(
                    TraceKernelBufferID::DirectionsBuffer,
                    e,
                ))
            }
        };
        let verticies_buffer = match Buffer::builder()
            .len(self.verticies.len())
            .queue(queue.clone())
            .copy_host_slice(&self.verticies)
            .build()
        {
            Ok(buffer) => buffer,
            Err(e) => {
                return Err(TraceKernelBuildError::BufferBuildError(
                    TraceKernelBufferID::VerticiesBuffer,
                    e,
                ))
            }
        };
        let triangles_buffer = match Buffer::builder()
            .len(self.triangles.len())
            .queue(queue.clone())
            .copy_host_slice(&self.triangles)
            .build()
        {
            Ok(buffer) => buffer,
            Err(e) => {
                return Err(TraceKernelBuildError::BufferBuildError(
                    TraceKernelBufferID::TrianglesBuffer,
                    e,
                ))
            }
        };
        let normals_buffer = match Buffer::builder()
            .len(self.normals.len())
            .queue(queue.clone())
            .copy_host_slice(&self.normals)
            .build()
        {
            Ok(buffer) => buffer,
            Err(e) => {
                return Err(TraceKernelBuildError::BufferBuildError(
                    TraceKernelBufferID::TrianglesBuffer,
                    e,
                ))
            }
        };
        match Kernel::builder()
            .program(&self.kernel_program)
            .arg(&output_image)
            .arg(&origins_buffer)
            .arg(&directions_buffer)
            .arg(&verticies_buffer)
            .arg(&triangles_buffer)
            .arg(&normals_buffer)
            .arg(&triangles_buffer.len())
            .build()
        {
            Ok(kernel) => Ok(TraceKernel {
                kernel: kernel,
                output_image: output_image,
                origins_buffer: origins_buffer,
                directions_buffer: directions_buffer,
                verticies_buffer: verticies_buffer,
                triangles_buffer: triangles_buffer,
                normals_buffer: normals_buffer,
            }),
            Err(e) => Err(TraceKernelBuildError::KernelBuildError(e)),
        }
    }
}
