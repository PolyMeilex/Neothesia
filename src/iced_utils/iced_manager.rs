pub struct IcedManager {
    pub renderer: iced_wgpu::Renderer,
    pub viewport: iced_wgpu::Viewport,
    pub debug: iced_native::Debug,
}

impl IcedManager {
    pub fn new(device: &wgpu::Device, physical_size: (u32, u32), scale_factor: f64) -> Self {
        let debug = iced_native::Debug::new();

        let settings = iced_wgpu::Settings::default();

        let renderer = iced_wgpu::Renderer::new(iced_wgpu::Backend::new(
            device,
            settings,
            wgpu_jumpstart::TEXTURE_FORMAT,
        ));

        let viewport = iced_wgpu::Viewport::with_physical_size(
            iced_native::Size::new(physical_size.0, physical_size.1),
            scale_factor,
        );

        Self {
            renderer,
            viewport,
            debug,
        }
    }

    pub fn resize(&mut self, physical_size: (u32, u32), scale_factor: f64) {
        self.viewport = iced_wgpu::Viewport::with_physical_size(
            iced_native::Size::new(physical_size.0, physical_size.1),
            scale_factor,
        );
    }
}
