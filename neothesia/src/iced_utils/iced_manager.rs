pub struct IcedManager {
    pub renderer: iced_wgpu::Renderer,
    pub viewport: iced_wgpu::graphics::Viewport,
    pub debug: iced_runtime::Debug,
    pub engine: iced_wgpu::Engine,
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
        let debug = iced_runtime::Debug::new();

        let engine = iced_wgpu::Engine::new(adapter, device, queue, texture_format, None);

        let renderer = iced_wgpu::Renderer::new(
            device,
            &engine,
            iced_core::Font::default(),
            iced_core::Pixels(16.0),
        );

        iced_graphics::text::font_system()
            .write()
            .expect("Write to font system")
            .load_font(std::borrow::Cow::Borrowed(include_bytes!(
                "./bootstrap-icons.ttf"
            )));

        let viewport = iced_wgpu::graphics::Viewport::with_physical_size(
            iced_core::Size::new(physical_size.0, physical_size.1),
            scale_factor,
        );

        Self {
            renderer,
            viewport,
            debug,
            engine,
        }
    }

    pub fn resize(&mut self, physical_size: (u32, u32), scale_factor: f64) {
        self.viewport = iced_wgpu::graphics::Viewport::with_physical_size(
            iced_core::Size::new(physical_size.0, physical_size.1),
            scale_factor,
        );
    }
}
