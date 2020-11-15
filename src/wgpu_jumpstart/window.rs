use super::{Gpu, GpuInitError};

pub struct Window {
    pub winit_window: winit::window::Window,

    surface: wgpu::Surface,

    swap_chain: wgpu::SwapChain,
    swap_chain_descriptor: wgpu::SwapChainDescriptor,
}

impl Window {
    pub async fn new(winit_window: winit::window::Window) -> Result<(Self, Gpu), GpuInitError> {
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
                .unwrap_or(Err(GpuInitError::AppendToBody)?);
        }

        let (gpu, surface) = Gpu::for_window(&winit_window).await?;

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

        Ok((
            Self {
                surface,
                winit_window,

                swap_chain,
                swap_chain_descriptor,
            },
            gpu,
        ))
    }

    #[inline]
    pub fn scale_factor(&self) -> f64 {
        self.winit_window.scale_factor()
    }

    #[inline]
    pub fn physical_size(&self) -> winit::dpi::PhysicalSize<u32> {
        self.winit_window.inner_size()
    }

    pub fn logical_size(&self) -> (f32, f32) {
        let ps = self.physical_size();
        let ls = ps.to_logical::<f32>(self.scale_factor());
        (ls.width, ls.height)
    }

    pub fn on_resize(&mut self, gpu: &mut Gpu) {
        let size = self.physical_size();

        self.swap_chain_descriptor.width = size.width;
        self.swap_chain_descriptor.height = size.height;

        self.swap_chain = gpu
            .device
            .create_swap_chain(&self.surface, &self.swap_chain_descriptor);
    }

    #[inline]
    pub fn request_redraw(&self) {
        self.winit_window.request_redraw();
    }

    #[inline]
    pub fn get_current_frame(&mut self) -> Result<wgpu::SwapChainFrame, wgpu::SwapChainError> {
        self.swap_chain.get_current_frame()
    }
}
