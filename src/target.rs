use std::cell::RefCell;
use std::rc::Rc;

use crate::config::Config;
use crate::input_manager::InputManager;
use crate::ui::TextRenderer;
use crate::utils::window::WindowState;
use crate::{EventLoopProxy, OutputManager, TransformUniform};
use wgpu_jumpstart::{Gpu, Uniform};

#[cfg(feature = "app")]
use crate::ui::IcedManager;
#[cfg(feature = "app")]
use winit::window::Window;

pub struct Target {
    #[cfg(feature = "app")]
    pub window: Window,
    #[cfg(feature = "app")]
    pub iced_manager: IcedManager,

    pub window_state: WindowState,
    pub gpu: Gpu,

    pub transform_uniform: Uniform<TransformUniform>,

    pub text_renderer: TextRenderer,

    pub output_manager: Rc<RefCell<OutputManager>>,
    pub input_manager: InputManager,
    pub midi_file: Option<Rc<lib_midi::Midi>>,
    pub config: Config,

    pub proxy: EventLoopProxy,
}

impl Target {
    pub fn new(
        #[cfg(feature = "app")] window: Window,
        window_state: WindowState,
        proxy: EventLoopProxy,
        gpu: Gpu,
    ) -> Self {
        let transform_uniform = Uniform::new(
            &gpu.device,
            TransformUniform::default(),
            wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
        );

        let text_renderer = TextRenderer::new(&gpu);

        #[cfg(feature = "app")]
        let iced_manager = IcedManager::new(
            &gpu.device,
            (
                window_state.physical_size.width,
                window_state.physical_size.height,
            ),
            window_state.scale_factor,
        );

        let args: Vec<String> = std::env::args().collect();

        let midi_file = if args.len() > 1 {
            if let Ok(midi) = lib_midi::Midi::new(&args[1]) {
                Some(Rc::new(midi))
            } else {
                None
            }
        } else {
            None
        };

        Self {
            #[cfg(feature = "app")]
            window,
            #[cfg(feature = "app")]
            iced_manager,

            window_state,
            gpu,
            transform_uniform,

            text_renderer,

            output_manager: Default::default(),
            input_manager: InputManager::new(proxy.clone()),
            midi_file,
            config: Config::new(),
            proxy,
        }
    }

    pub fn resize(&mut self) {
        self.transform_uniform.data.update(
            self.window_state.logical_size.width,
            self.window_state.logical_size.height,
            self.window_state.scale_factor as f32,
        );
        self.transform_uniform.update(&self.gpu.queue);

        #[cfg(feature = "app")]
        self.iced_manager.resize(
            (
                self.window_state.physical_size.width,
                self.window_state.physical_size.height,
            ),
            self.window_state.scale_factor,
        );
    }
}
