use super::gpu::Gpu;
use super::surface::Surface;

use winit::event_loop::EventLoop;
use winit::window::WindowBuilder;

pub struct Window {
    pub surface: Surface,
    pub winit_window: winit::window::Window,
    pub width: u32,
    pub height: u32,
    pub dpi: f64,
}

impl Window {
    pub async fn new(
        builder: WindowBuilder,
        size: (u32, u32),
        event_loop: &EventLoop<()>,
    ) -> (Self, Gpu) {
        let dpi = event_loop.primary_monitor().unwrap().scale_factor();

        let (width, height) = size;

        let width = (width as f64 / dpi).round();
        let height = (height as f64 / dpi).round();

        let winit_window = builder
            .with_inner_size(winit::dpi::LogicalSize { width, height })
            .build(event_loop)
            .unwrap();

        #[cfg(target_arch = "wasm32")]
        {
            use winit::platform::web::WindowExtWebSys;
            web_sys::window()
                .and_then(|win| win.document())
                .and_then(|doc| doc.body())
                .and_then(|body| {
                    body.append_child(&web_sys::Element::from(winit_window.canvas()))
                        .ok()
                })
                .expect("couldn't append canvas to document body");
        }

        let (gpu, surface) = Gpu::for_window(&winit_window).await;

        let size = winit_window.inner_size();

        (
            Self {
                surface,
                winit_window,
                width: size.width,
                height: size.height,
                dpi,
            },
            gpu,
        )
    }
    pub fn size(&self) -> (f32, f32) {
        (
            self.width as f32 / self.dpi as f32,
            self.height as f32 / self.dpi as f32,
        )
    }
    pub fn physical_size(&self) -> winit::dpi::PhysicalSize<u32> {
        self.winit_window.inner_size()
    }
    pub fn on_resize(&mut self, gpu: &mut Gpu) {
        self.surface.resize(gpu, self.physical_size());
        let size = self.winit_window.inner_size();
        self.width = size.width;
        self.height = size.height;
    }
    pub fn on_dpi(&mut self, dpi: f64) {
        self.dpi = dpi;
    }
    pub fn request_redraw(&self) {
        self.winit_window.request_redraw();
    }
}
