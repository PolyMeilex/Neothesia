use crate::{
    image::{
        self,
        atlas::{self, Atlas},
    },
    Size,
};

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

    pub fn measure_image(&mut self, handle: &image::Handle) -> Size<u32> {
        self.raster.load(handle).dimensions()
    }

    pub fn upload_raster(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        handle: &image::Handle,
    ) -> Option<&atlas::Entry> {
        self.raster.upload(device, encoder, handle, &mut self.atlas)
    }

    pub fn trim(&mut self) {
        self.raster.trim(&mut self.atlas);
    }
}
