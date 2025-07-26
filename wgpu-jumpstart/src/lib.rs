#![allow(clippy::single_match)]

mod error;
use std::ops::{Deref, DerefMut};

pub use error::GpuInitError;

mod color;
mod gpu;
mod instances;
mod render_pipeline_builder;
mod shape;
mod uniform;

mod transform_uniform;

pub use wgpu;
pub use {
    color::Color,
    gpu::{Gpu, Surface},
    instances::Instances,
    render_pipeline_builder::{
        default_color_target_state, default_fragment, default_render_pipeline, default_vertex,
    },
    shape::Shape,
    transform_uniform::TransformUniform,
    uniform::Uniform,
};

pub struct RenderPass<'a>(wgpu::RenderPass<'a>, wgpu::Extent3d);

impl<'a> RenderPass<'a> {
    pub fn new(rpass: wgpu::RenderPass<'a>, size: wgpu::Extent3d) -> Self {
        Self(rpass, size)
    }

    pub fn size(&self) -> wgpu::Extent3d {
        self.1
    }
}

impl<'a> Deref for RenderPass<'a> {
    type Target = wgpu::RenderPass<'a>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> DerefMut for RenderPass<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
