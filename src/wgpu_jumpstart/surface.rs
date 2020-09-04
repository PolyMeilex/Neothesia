pub struct Surface {
    surface: wgpu::Surface,
    swap_chain: wgpu::SwapChain,
    swap_chain_descriptor: wgpu::SwapChainDescriptor,
}

impl Surface {
    pub fn new(
        window: &winit::window::Window,
        surface: wgpu::Surface,
        device: &wgpu::Device,
    ) -> Self {
        let size = window.inner_size();

        let swap_chain_descriptor = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            format: super::TEXTURE_FORMAT,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };

        let swap_chain = device.create_swap_chain(&surface, &swap_chain_descriptor);

        Self {
            surface,
            swap_chain,
            swap_chain_descriptor,
        }
    }
    pub fn resize(&mut self, gpu: &mut super::gpu::Gpu, size: winit::dpi::PhysicalSize<u32>) {
        self.swap_chain_descriptor.width = size.width;
        self.swap_chain_descriptor.height = size.height;

        self.swap_chain = gpu
            .device
            .create_swap_chain(&self.surface, &self.swap_chain_descriptor);
    }
    pub fn get_current_frame(&mut self) -> wgpu::SwapChainFrame {
        self.swap_chain
            .get_current_frame()
            .expect("Surface::get_current_frame")
    }
}
