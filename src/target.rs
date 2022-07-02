use winit::event_loop::EventLoopProxy;

use crate::config::Config;
use crate::ui::{self, TextRenderer};
use crate::{NeothesiaEvent, OutputManager, TransformUniform};
use wgpu_jumpstart::{Gpu, Uniform, Window};

pub struct Target {
    pub window: Window,
    pub gpu: Gpu,
    pub transform_uniform: Uniform<TransformUniform>,

    pub text_renderer: TextRenderer,
    #[cfg(feature = "app")]
    pub iced_manager: ui::IcedManager,

    pub output_manager: OutputManager,
    pub midi_file: Option<lib_midi::Midi>,
    pub config: Config,

    pub proxy: EventLoopProxy<NeothesiaEvent>,
}

impl Target {
    pub fn new(window: Window, proxy: EventLoopProxy<NeothesiaEvent>, gpu: Gpu) -> Self {
        let transform_uniform = Uniform::new(
            &gpu.device,
            TransformUniform::default(),
            wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
        );

        let text_renderer = TextRenderer::new(&gpu);

        #[cfg(feature = "app")]
        let iced_manager = ui::IcedManager::new(&gpu.device, &window);

        let args: Vec<String> = std::env::args().collect();

        let midi_file = if args.len() > 1 {
            if let Ok(midi) = lib_midi::Midi::new(&args[1]) {
                Some(midi)
            } else {
                None
            }
        } else {
            None
        };

        Self {
            window,
            gpu,
            transform_uniform,

            text_renderer,
            #[cfg(feature = "app")]
            iced_manager,

            output_manager: Default::default(),
            midi_file,
            config: Config::new(),
            proxy,
        }
    }

    pub fn resize(&mut self) {
        {
            let winit::dpi::LogicalSize { width, height } = self.window.state.logical_size;
            self.transform_uniform.data.update(width, height);
            self.transform_uniform.update(&self.gpu.queue);
        }

        #[cfg(feature = "app")]
        {
            let physical_size = self.window.state.physical_size;
            self.iced_manager.viewport = iced_wgpu::Viewport::with_physical_size(
                iced_native::Size::new(physical_size.width, physical_size.height),
                self.window.state.scale_factor,
            );
        }
    }
}
