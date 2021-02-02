use crate::wgpu_jumpstart;
use crate::Window;

pub struct IcedManager {
    pub renderer: iced_wgpu::Renderer,
    pub viewport: iced_wgpu::Viewport,
    pub debug: iced_native::Debug,
}
impl IcedManager {
    pub fn new(device: &wgpu::Device, window: &Window) -> Self {
        let debug = iced_native::Debug::new();

        let settings = iced_wgpu::Settings {
            format: wgpu_jumpstart::TEXTURE_FORMAT,
            ..Default::default()
        };

        let renderer = iced_wgpu::Renderer::new(iced_wgpu::Backend::new(device, settings));

        let physical_size = window.state.physical_size;
        let viewport = iced_wgpu::Viewport::with_physical_size(
            iced_native::Size::new(physical_size.width, physical_size.height),
            window.state.scale_factor,
        );

        Self {
            renderer,
            viewport,
            debug,
        }
    }
}
