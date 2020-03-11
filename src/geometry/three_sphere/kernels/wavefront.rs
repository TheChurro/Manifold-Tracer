use crate::geometry::three_sphere::kernels::shade_kernel::ShadeKernelBuildError;
use crate::geometry::three_sphere::kernels::trace_kernel::TraceKernelBuildError;
use crate::geometry::three_sphere::kernels::{SampleAggregator, ShadeKernel, TraceKernel};
use crate::geometry::three_sphere::object::MaterialType;
use crate::geometry::three_sphere::BoundingVolumeHierarchy;
use crate::geometry::three_sphere::Point;
use crate::geometry::three_sphere::{Ball, Object, Triangle};

use ocl::{enums::DeviceSpecifier, Device, DeviceType};
use ocl::{Context, Platform, Queue};

use ocl::{
    prm::{Float4, Uint4},
    Buffer,
};
/// The set of buffers which alternate as in and out buffers
/// for the trace and shade kernels. trace_* buffers are inputs
/// to the trace kernel and outputs for the shade kernel. Similarly,
/// shade_* buffers are inputs to the shade kernel and outputs for
/// the trace kernel.
pub struct InOutBufferSet {
    pub sample_color_out: Buffer<Float4>,

    pub trace_ray_origin: Buffer<Float4>,
    pub trace_ray_tangent: Buffer<Float4>,
    pub trace_ray_color: Buffer<Float4>,
    pub trace_ray_info: Buffer<Uint4>,

    pub hit_normal: Buffer<Float4>,

    pub shade_ray_origin: Buffer<Float4>,
    pub shade_ray_tangent: Buffer<Float4>,
    pub shade_ray_color: Buffer<Float4>,
    pub shade_ray_info: Buffer<Uint4>,
    pub shade_rays_produced: Buffer<u32>,
}

#[derive(Clone, Copy, Debug)]
pub enum InOutBufferID {
    SampleColorOutBuffer,
    TraceRayOrigin,
    TraceRayTangent,
    TraceRayColor,
    TraceRayInfo,
    HitNormals,
    ShadeRayOrigin,
    ShadeRayTangent,
    ShadeRayColor,
    ShadeRayInfo,
    ShadeRaysProduced,
}
#[derive(Debug)]
pub enum WavefrontBuildError {
    BufferBuildError {
        buffer: InOutBufferID,
        error: ocl::Error,
    },
    TraceKernelBuildError(TraceKernelBuildError),
    TraceKernelError(ocl::Error),
    ShadeKernelBuildError(ShadeKernelBuildError),
    ShadeKernelError(ocl::Error),
    AggregateKernelBuildError(ocl::Error),
    AggregateKernelError(ocl::Error),
}

use std::fmt::{Display, Error as FmtError, Formatter};
impl Display for WavefrontBuildError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        use WavefrontBuildError::*;
        match &self {
            &BufferBuildError {
                ref buffer,
                ref error,
            } => write!(f, "Failed to build buffer {:?} due to {:?}", buffer, error),
            &TraceKernelBuildError(ref e) => write!(f, "Trace Kernel Build Error: {}", e),
            &TraceKernelError(ref e) => write!(f, "Trace Kernel Error: {}", e),
            &ShadeKernelBuildError(ref e) => write!(f, "Shade Kernel Build Error: {}", e),
            &ShadeKernelError(ref e) => write!(f, "Shade Kernel Error: {}", e),
            &AggregateKernelBuildError(ref e) => write!(f, "Aggregate Kernel Build Error: {}", e),
            &AggregateKernelError(ref e) => write!(f, "Aggregate Kernel Error: {}", e),
        }
    }
}

impl InOutBufferSet {
    fn build_buffer<T: ocl::OclPrm>(
        len: u32,
        buffer_id: InOutBufferID,
        queue: &Queue,
    ) -> Result<Buffer<T>, WavefrontBuildError> {
        match Buffer::builder().len(len).queue(queue.clone()).build() {
            Ok(buffer) => Ok(buffer),
            Err(e) => Err(WavefrontBuildError::BufferBuildError {
                buffer: buffer_id,
                error: e,
            }),
        }
    }

    pub fn new(num_rays: u32, queue: &Queue) -> Result<Self, WavefrontBuildError> {
        use InOutBufferID::*;
        let t_ray_origin = Self::build_buffer(num_rays, TraceRayOrigin, queue)?;
        let t_ray_tangent = Self::build_buffer(num_rays, TraceRayTangent, queue)?;
        let t_ray_color = Self::build_buffer(num_rays, TraceRayColor, queue)?;
        let t_ray_info = Self::build_buffer(num_rays, TraceRayInfo, queue)?;

        let hit_normal = Self::build_buffer(num_rays, HitNormals, queue)?;

        let s_ray_origin = Self::build_buffer(num_rays, ShadeRayOrigin, queue)?;
        let s_ray_tangent = Self::build_buffer(num_rays, ShadeRayTangent, queue)?;
        let s_ray_color = Self::build_buffer(num_rays, ShadeRayColor, queue)?;
        let s_ray_info = Self::build_buffer(num_rays, ShadeRayInfo, queue)?;
        let s_rays_out = Self::build_buffer(2, ShadeRaysProduced, queue)?;

        let sample_color_out = Self::build_buffer(num_rays, SampleColorOutBuffer, queue)?;
        let zero_color = vec![Float4::new(0.0, 0.0, 0.0, 1.0); num_rays as usize];
        sample_color_out.write(&zero_color).enq().map_err(|e| {
            WavefrontBuildError::BufferBuildError {
                buffer: SampleColorOutBuffer,
                error: e,
            }
        })?;
        Ok(Self {
            sample_color_out: sample_color_out,
            trace_ray_origin: t_ray_origin,
            trace_ray_tangent: t_ray_tangent,
            trace_ray_color: t_ray_color,
            trace_ray_info: t_ray_info,
            hit_normal: hit_normal,
            shade_ray_origin: s_ray_origin,
            shade_ray_tangent: s_ray_tangent,
            shade_ray_color: s_ray_color,
            shade_ray_info: s_ray_info,
            shade_rays_produced: s_rays_out,
        })
    }
}

struct BuildInfo {
    pub num_rays: u32,
    pub width: u32,
    pub height: u32,
    pub num_samples: u32,
}

struct BuiltShaders {
    trace: TraceKernel,
    shade: ShadeKernel,
    aggregate: SampleAggregator,
    buffer_set: InOutBufferSet,
    info: BuildInfo,
}

pub struct Wavefront {
    pub device: Device,
    pub context: Context,
    pub queue: Queue,
    pub triangles: Vec<Object<Triangle>>,
    pub balls: Vec<Object<Ball>>,
    shaders: Option<BuiltShaders>,
}

impl Wavefront {
    pub fn new() -> Wavefront {
        // Grab the first GPU from the device list.
        let device = {
            match DeviceSpecifier::from(DeviceType::GPU).to_device_list(Some(Platform::default())) {
                Ok(devices) => {
                    if devices.len() == 0 {
                        panic!("No GPU devices found!");
                    }
                    if devices.len() > 0 {
                        println!("Found multiple GPUs! Defaulting to first found!");
                    }
                    devices[0]
                }
                Err(e) => {
                    panic!("Failed to load GPU devices: {}", e);
                }
            }
        };
        let context = match Context::builder()
            .platform(Platform::default())
            .devices(device)
            .build()
        {
            Ok(context) => context,
            Err(e) => panic!("Failed to create contex: {}", e),
        };
        let queue = match Queue::new(&context, device, None) {
            Ok(queue) => queue,
            Err(e) => panic!("Failed to create queue: {}", e),
        };
        Wavefront {
            device: device,
            context: context,
            queue: queue,
            triangles: Vec::new(),
            balls: Vec::new(),
            shaders: None,
        }
    }

    pub fn ball(&mut self, ball: Ball, color: [f32; 3], material: MaterialType) {
        self.balls.push(Object {
            geometry: ball,
            color: color,
            material: material,
        });
    }

    pub fn triangle(&mut self, triangle: Triangle, color: [f32; 3], material: MaterialType) {
        self.triangles.push(Object {
            geometry: triangle,
            color: color,
            material: material,
        });
    }

    pub fn build(
        &mut self,
        width: u32,
        height: u32,
        num_samples: u32,
    ) -> Result<(), WavefrontBuildError> {
        let info = BuildInfo {
            num_rays: width * height * num_samples,
            width: width,
            height: height,
            num_samples: num_samples,
        };
        let buffers = InOutBufferSet::new(info.num_rays, &self.queue)?;
        let mut trace_builder = match TraceKernel::builder(&self.context, &self.device) {
            Ok(builder) => builder,
            Err(e) => return Err(WavefrontBuildError::TraceKernelError(e)),
        };
        let mut shade_builder = match ShadeKernel::builder(&self.context, &self.device) {
            Ok(builder) => builder,
            Err(e) => return Err(WavefrontBuildError::ShadeKernelError(e)),
        };
        let accelerator = BoundingVolumeHierarchy::new(&mut self.triangles, &mut self.balls);
        for tri_obj in &self.triangles {
            trace_builder.load_triangle(&tri_obj.geometry);
            shade_builder.add_triangle_material(tri_obj.color, tri_obj.material);
        }
        for ball_obj in &self.balls {
            trace_builder.load_ball(&ball_obj.geometry);
            shade_builder.add_ball_material(ball_obj.color, ball_obj.material);
        }
        let trace = match trace_builder.build(info.num_rays, &self.queue, &buffers, &accelerator) {
            Ok(trace) => trace,
            Err(e) => return Err(WavefrontBuildError::TraceKernelBuildError(e)),
        };
        let shade = match shade_builder.build(info.num_rays, &buffers, &self.queue) {
            Ok(shade) => shade,
            Err(e) => return Err(WavefrontBuildError::ShadeKernelBuildError(e)),
        };
        let aggregate = match SampleAggregator::new(
            info.width,
            info.height,
            info.num_samples,
            &buffers,
            &self.device,
            &self.context,
            &self.queue,
        ) {
            Ok(aggregate) => aggregate,
            Err(e) => return Err(WavefrontBuildError::AggregateKernelBuildError(e)),
        };
        self.shaders = Some(BuiltShaders {
            trace: trace,
            shade: shade,
            aggregate: aggregate,
            buffer_set: buffers,
            info: info,
        });
        Ok(())
    }

    pub fn run(
        &mut self,
        rays: Vec<(Point, Point)>,
        img: &mut image::RgbaImage,
    ) -> Result<(), ocl::Error> {
        if let &mut Some(ref mut shaders) = &mut self.shaders {
            use rand::RngCore;
            let mut rng = rand::thread_rng();
            let mut ray_origins: Vec<Float4> = Vec::new();
            let mut ray_tangents: Vec<Float4> = Vec::new();
            let mut ray_colors: Vec<Float4> = Vec::new();
            let mut ray_info: Vec<Uint4> = Vec::new();
            for (i, ray) in rays.iter().enumerate() {
                ray_origins.push(ray.0.into());
                ray_tangents.push(ray.1.into());
                ray_colors.push([1.0; 4].into());
                ray_info.push(Uint4::new(0, 0, i as u32, rng.next_u32()));
            }
            shaders
                .buffer_set
                .trace_ray_origin
                .write(&ray_origins)
                .enq()?;
            shaders
                .buffer_set
                .trace_ray_tangent
                .write(&ray_tangents)
                .enq()?;
            shaders
                .buffer_set
                .trace_ray_color
                .write(&ray_colors)
                .enq()?;
            shaders.buffer_set.trace_ray_info.write(&ray_info).enq()?;
            let mut num_rays = shaders.info.num_rays;
            // Trace rays a number of times
            for i in 0..10 {
                shaders
                    .buffer_set
                    .shade_rays_produced
                    .write(&vec![0, i])
                    .enq()?;
                if num_rays == 0 {
                    break;
                }
                shaders.trace.run(num_rays)?;
                shaders.shade.shade(num_rays)?;
                let mut num_rays_buf = vec![0];
                shaders
                    .buffer_set
                    .shade_rays_produced
                    .read(&mut num_rays_buf)
                    .enq()?;
                num_rays = num_rays_buf[0] as u32;
            }
            shaders.aggregate.run(img)?;
        }
        Ok(())
    }
}
