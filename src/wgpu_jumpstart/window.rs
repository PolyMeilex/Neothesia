use super::gpu::Gpu;
use super::surface::Surface;

use winit::event_loop::EventLoop;
use winit::window::WindowBuilder;

pub struct Window {
    pub surface: Surface,
    pub winit_window: winit::window::Window,
    pub width: f32,
    pub height: f32,
    pub dpi: f64,
}

impl Window {
    pub async fn new(
        builder: WindowBuilder,
        size: (u32, u32),
        event_loop: &EventLoop<()>,
    ) -> (Self, Gpu) {
        let dpi = event_loop.primary_monitor().scale_factor();

        let (width, height) = size;

        let width = (width as f64 / dpi).round();
        let height = (height as f64 / dpi).round();

        let winit_window = builder
            .with_inner_size(winit::dpi::LogicalSize { width, height })
            .build(event_loop)
            .unwrap();

        let (gpu, surface) = Gpu::for_window(&winit_window).await;

        let size = winit_window.inner_size();

        (
            Self {
                surface,
                winit_window,
                width: size.width as f32,
                height: size.height as f32,
                dpi,
            },
            gpu,
        )
    }
    pub fn size(&self) -> (f32, f32) {
        let size = self.winit_window.inner_size();
        (size.width as f32, size.height as f32)
    }
    pub fn physical_size(&self) -> winit::dpi::PhysicalSize<u32> {
        self.winit_window.inner_size()
    }
    pub fn resize(&mut self, gpu: &mut Gpu) {
        let new_size = self.winit_window.inner_size();
        self.surface.resize(gpu, new_size);

        self.width = new_size.width as f32;
        self.height = new_size.height as f32;
    }
    pub fn request_redraw(&self) {
        self.winit_window.request_redraw();
    }
}
