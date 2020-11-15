mod error;
pub use error::GpuInitError;

mod color;
mod gpu;
mod instances;
mod render_pipeline_builder;
mod shape;
mod uniform;
mod window;

pub use {
    color::Color, gpu::Gpu, instances::Instances, render_pipeline_builder::RenderPipelineBuilder,
    shape::Shape, uniform::Uniform, window::Window,
};

pub const TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8UnormSrgb;
