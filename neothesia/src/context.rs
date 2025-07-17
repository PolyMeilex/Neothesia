use std::sync::Arc;

use crate::config::Config;
use crate::input_manager::InputManager;
use crate::utils::window::WindowState;
use crate::{output_manager::OutputManager, NeothesiaEvent, TransformUniform};
use neothesia_core::utils::fps_ticker;
use wgpu_jumpstart::{Gpu, Uniform};
use winit::event_loop::EventLoopProxy;

use crate::iced_utils::IcedManager;
use winit::window::Window;

pub struct Context {
    pub window: Arc<Window>,
    pub iced_manager: IcedManager,

    pub window_state: WindowState,
    pub gpu: Gpu,

    pub transform: Uniform<TransformUniform>,

    pub output_manager: OutputManager,
    pub input_manager: InputManager,
    pub config: Config,

    pub proxy: EventLoopProxy<NeothesiaEvent>,

    /// Last frame timestamp
    pub frame_timestamp: std::time::Instant,

    #[cfg(debug_assertions)]
    pub fps_ticker: fps_ticker::Fps,
}

impl Drop for Context {
    fn drop(&mut self) {
        self.config.save();
    }
}

impl Context {
    pub fn new(
        window: Arc<Window>,
        window_state: WindowState,
        proxy: EventLoopProxy<NeothesiaEvent>,
        gpu: Gpu,
    ) -> Self {
        let transform_uniform = Uniform::new(
            &gpu.device,
            TransformUniform::default(),
            wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
        );

        let iced_manager = IcedManager::new(
            &gpu.adapter,
            &gpu.device,
            &gpu.queue,
            gpu.texture_format,
            (
                window_state.physical_size.width,
                window_state.physical_size.height,
            ),
            window_state.scale_factor,
        );

        let config = Config::new();

        Self {
            window,
            iced_manager,

            window_state,
            gpu,
            transform: transform_uniform,

            output_manager: Default::default(),
            input_manager: InputManager::new(proxy.clone()),
            config,
            proxy,
            frame_timestamp: std::time::Instant::now(),

            #[cfg(debug_assertions)]
            fps_ticker: fps_ticker::Fps::default(),
        }
    }

    pub fn resize(&mut self) {
        self.transform.data.update(
            self.window_state.logical_size.width,
            self.window_state.logical_size.height,
            self.window_state.scale_factor as f32,
        );
        self.transform.update(&self.gpu.queue);

        self.iced_manager.resize(
            (
                self.window_state.physical_size.width,
                self.window_state.physical_size.height,
            ),
            self.window_state.scale_factor,
        );
    }
}
