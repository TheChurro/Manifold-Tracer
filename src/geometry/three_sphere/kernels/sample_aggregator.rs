use super::wavefront::InOutBufferSet;
use super::AGGREGATE_KERNEL;

use ocl::error::Error as OclError;
use ocl::{
    enums::{ImageChannelDataType, ImageChannelOrder, MemObjectType},
    flags::{MEM_HOST_READ_ONLY, MEM_WRITE_ONLY},
    Image,
};

use ocl::{Context, Device, Kernel, Program, Queue};
use std::ffi::CString;

pub struct SampleAggregator {
    kernel: Kernel,
    out_image: Image<u8>,
}

impl SampleAggregator {
    pub fn new(
        width: u32,
        height: u32,
        num_samples: u32,
        in_out_buffers: &InOutBufferSet,
        device: &Device,
        context: &Context,
        queue: &Queue,
    ) -> Result<SampleAggregator, OclError> {
        let program_source = CString::new(AGGREGATE_KERNEL)
            .unwrap_or_else(|e| panic!("Failed to turn trace kernel into CString: {}", e));
        let program = Program::with_source(
            context,
            &[program_source],
            Some(&[device.clone()]),
            &CString::default(),
        )?;
        let output_image = match Image::<u8>::builder()
            .dims((width, height))
            .channel_order(ImageChannelOrder::Rgba)
            .channel_data_type(ImageChannelDataType::UnormInt8)
            .image_type(MemObjectType::Image2d)
            .flags(MEM_WRITE_ONLY | MEM_HOST_READ_ONLY)
            .queue(queue.clone())
            .build()
        {
            Ok(img) => img,
            Err(e) => return Err(e),
        };
        let n_samples: i32 = num_samples as i32;
        let kernel = Kernel::builder()
            .program(&program)
            .name("aggregate")
            .global_work_size((width, height))
            .arg(&output_image)
            .arg(&in_out_buffers.sample_color_out)
            .arg(&n_samples)
            .queue(queue.clone())
            .build()?;
        Ok(SampleAggregator {
            kernel: kernel,
            out_image: output_image,
        })
    }

    pub fn run(&mut self, img: &mut image::RgbaImage) -> Result<(), OclError> {
        unsafe { self.kernel.enq()? };
        self.out_image.read(img).enq()?;
        Ok(())
    }
}
