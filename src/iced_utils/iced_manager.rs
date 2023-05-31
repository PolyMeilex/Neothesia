pub struct IcedManager {
    pub renderer: iced_wgpu::Renderer<iced_style::Theme>,
    pub viewport: iced_wgpu::graphics::Viewport,
    pub debug: iced_runtime::Debug,
}

impl IcedManager {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        physical_size: (u32, u32),
        scale_factor: f64,
    ) -> Self {
        let debug = iced_runtime::Debug::new();

        let settings = iced_wgpu::Settings::default();

        let renderer = iced_wgpu::Renderer::new(iced_wgpu::Backend::new(
            device,
            queue,
            settings,
            wgpu_jumpstart::TEXTURE_FORMAT,
        ));

        let viewport = iced_wgpu::graphics::Viewport::with_physical_size(
            iced_core::Size::new(physical_size.0, physical_size.1),
            scale_factor,
        );

        Self {
            renderer,
            viewport,
            debug,
        }
    }

    pub fn resize(&mut self, physical_size: (u32, u32), scale_factor: f64) {
        self.viewport = iced_wgpu::graphics::Viewport::with_physical_size(
            iced_core::Size::new(physical_size.0, physical_size.1),
            scale_factor,
        );
    }
}
