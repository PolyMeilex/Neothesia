use std::sync::Arc;

use crate::config::Config;
use crate::input_manager::InputManager;
use crate::utils::window::WindowState;
use crate::{output_manager::OutputManager, NeothesiaEvent, TransformUniform};
use neothesia_core::render::{QuadRendererFactory, TextRendererFactory};
use neothesia_core::Size;
use wgpu_jumpstart::{Gpu, Uniform};
use winit::event_loop::EventLoopProxy;

use winit::window::Window;

pub struct Context {
    pub window: Arc<Window>,
    pub iced_renderer: iced_wgpu::Renderer,

    pub window_state: WindowState,
    pub gpu: Gpu,

    pub transform: Uniform<TransformUniform>,
    pub text_renderer_factory: TextRendererFactory,
    pub quad_renderer_facotry: QuadRendererFactory,

    pub output_manager: OutputManager,
    pub input_manager: InputManager,
    pub config: Config,

    pub proxy: EventLoopProxy<NeothesiaEvent>,

    /// Last frame timestamp
    pub frame_timestamp: std::time::Instant,

    #[cfg(debug_assertions)]
    pub fps_ticker: neothesia_core::utils::fps_ticker::Fps,
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

        let iced_renderer = iced_wgpu::Renderer::new(
            &gpu.adapter,
            gpu.device.clone(),
            gpu.queue.clone(),
            gpu.texture_format,
            Size::new(
                window_state.physical_size.width,
                window_state.physical_size.height,
            ),
            window_state.scale_factor,
        );

        let config = Config::new();

        let text_renderer_factory = TextRendererFactory::new(&gpu);
        let quad_renderer_facotry = QuadRendererFactory::new(&gpu, &transform_uniform);

        Self {
            window,
            iced_renderer,

            window_state,
            gpu,
            transform: transform_uniform,
            text_renderer_factory,
            quad_renderer_facotry,

            output_manager: Default::default(),
            input_manager: InputManager::new(proxy.clone()),
            config,
            proxy,
            frame_timestamp: std::time::Instant::now(),

            #[cfg(debug_assertions)]
            fps_ticker: Default::default(),
        }
    }

    pub fn resize(&mut self) {
        self.transform.data.update(
            self.window_state.physical_size.width as f32,
            self.window_state.physical_size.height as f32,
            self.window_state.scale_factor as f32,
        );
        self.transform.update(&self.gpu.queue);

        self.iced_renderer.resize(
            Size::new(
                self.window_state.physical_size.width,
                self.window_state.physical_size.height,
            ),
            self.window_state.scale_factor,
        );
    }
}
