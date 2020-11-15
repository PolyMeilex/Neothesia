mod color;
mod gpu;
mod instances;
mod render_pipeline_builder;
mod simple_quad;
mod uniform;
mod window;

pub mod shader;
pub use {
    color::Color, gpu::Gpu, instances::Instances, render_pipeline_builder::RenderPipelineBuilder,
    simple_quad::SimpleQuad, uniform::Uniform, window::Window,
};

pub const TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8UnormSrgb;
