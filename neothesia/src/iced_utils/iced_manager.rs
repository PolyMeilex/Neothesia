pub struct IcedManager {
    pub renderer: iced_wgpu::Renderer,
    pub viewport: iced_wgpu::graphics::Viewport,
}

impl IcedManager {
    pub fn new(
        adapter: &wgpu::Adapter,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture_format: wgpu::TextureFormat,
        physical_size: (u32, u32),
        scale_factor: f64,
    ) -> Self {
        let engine =
            iced_wgpu::Engine::new(adapter, device.clone(), queue.clone(), texture_format, None);

        let renderer = iced_wgpu::Renderer::new(
            engine,
            iced_core::Font::with_name("Roboto"),
            iced_core::Pixels(16.0),
        );

        let viewport = iced_wgpu::graphics::Viewport::with_physical_size(
            iced_core::Size::new(physical_size.0, physical_size.1),
            scale_factor,
        );

        Self { renderer, viewport }
    }

    pub fn resize(&mut self, physical_size: (u32, u32), scale_factor: f64) {
        self.viewport = iced_wgpu::graphics::Viewport::with_physical_size(
            iced_core::Size::new(physical_size.0, physical_size.1),
            scale_factor,
        );
    }
}
