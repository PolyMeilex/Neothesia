pub struct Surface {
    surface: wgpu::Surface,
    swap_chain: wgpu::SwapChain,
}

impl Surface {
    pub fn new(
        window: &winit::window::Window,
        surface: wgpu::Surface,
        device: &wgpu::Device,
    ) -> Self {
        let size = window.inner_size();

        let swap_chain = device.create_swap_chain(
            &surface,
            &wgpu::SwapChainDescriptor {
                usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
                format: wgpu::TextureFormat::Bgra8Unorm,
                width: size.width,
                height: size.height,
                present_mode: wgpu::PresentMode::Fifo,
            },
        );

        Self {
            surface,
            swap_chain,
        }
    }
    pub fn resize(&mut self, gpu: &mut super::gpu::Gpu, size: winit::dpi::PhysicalSize<u32>) {
        self.swap_chain = gpu.device.create_swap_chain(
            &self.surface,
            &wgpu::SwapChainDescriptor {
                usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
                format: wgpu::TextureFormat::Bgra8Unorm,
                width: size.width,
                height: size.height,
                present_mode: wgpu::PresentMode::Fifo,
            },
        );
    }
    pub fn get_next_texture(&mut self) -> wgpu::SwapChainOutput {
        self.swap_chain
            .get_next_texture()
            .expect("get_next_texture")
    }
}
