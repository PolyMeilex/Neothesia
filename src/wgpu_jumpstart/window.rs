use super::gpu::Gpu;

use winit::event_loop::EventLoop;
use winit::window::WindowBuilder;

pub struct Window {
    pub winit_window: winit::window::Window,
    pub dpi: f64,

    surface: wgpu::Surface,

    swap_chain: wgpu::SwapChain,
    swap_chain_descriptor: wgpu::SwapChainDescriptor,
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

        let (swap_chain, swap_chain_descriptor) = {
            let size = winit_window.inner_size();

            let swap_chain_descriptor = wgpu::SwapChainDescriptor {
                usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
                format: super::TEXTURE_FORMAT,
                width: size.width,
                height: size.height,
                present_mode: wgpu::PresentMode::Fifo,
            };

            let swap_chain = gpu
                .device
                .create_swap_chain(&surface, &swap_chain_descriptor);
            (swap_chain, swap_chain_descriptor)
        };

        (
            Self {
                surface,
                winit_window,
                dpi,

                swap_chain,
                swap_chain_descriptor,
            },
            gpu,
        )
    }

    pub fn physical_size(&self) -> winit::dpi::PhysicalSize<u32> {
        self.winit_window.inner_size()
    }

    pub fn size(&self) -> (f32, f32) {
        let ps = self.physical_size();
        (
            ps.width as f32 / self.dpi as f32,
            ps.height as f32 / self.dpi as f32,
        )
    }

    pub fn on_resize(&mut self, gpu: &mut Gpu) {
        let size = self.physical_size();

        self.swap_chain_descriptor.width = size.width;
        self.swap_chain_descriptor.height = size.height;

        self.swap_chain = gpu
            .device
            .create_swap_chain(&self.surface, &self.swap_chain_descriptor);
    }

    pub fn on_dpi(&mut self, dpi: f64) {
        self.dpi = dpi;
    }

    pub fn request_redraw(&self) {
        self.winit_window.request_redraw();
    }

    pub fn get_current_frame(&mut self) -> wgpu::SwapChainFrame {
        self.swap_chain
            .get_current_frame()
            .expect("Surface::get_current_frame")
    }
}
