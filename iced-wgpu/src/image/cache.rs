use crate::core::{self, Size};
use crate::image::atlas::{self, Atlas};

use std::sync::Arc;

#[derive(Debug)]
pub struct Cache {
    atlas: Atlas,
    raster: crate::image::raster::Cache,
}

impl Cache {
    pub fn new(
        device: &wgpu::Device,
        backend: wgpu::Backend,
        layout: Arc<wgpu::BindGroupLayout>,
    ) -> Self {
        Self {
            atlas: Atlas::new(device, backend, layout),
            raster: crate::image::raster::Cache::default(),
        }
    }

    pub fn bind_group(&self) -> &wgpu::BindGroup {
        self.atlas.bind_group()
    }

    pub fn layer_count(&self) -> usize {
        self.atlas.layer_count()
    }

    pub fn measure_image(&mut self, handle: &core::image::Handle) -> Size<u32> {
        self.raster.load(handle).dimensions()
    }

    pub fn upload_raster(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        handle: &core::image::Handle,
    ) -> Option<&atlas::Entry> {
        self.raster.upload(device, encoder, handle, &mut self.atlas)
    }

    pub fn trim(&mut self) {
        self.raster.trim(&mut self.atlas);
    }
}
