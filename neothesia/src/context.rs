use std::sync::Arc;

use crate::{
    NeothesiaEvent, TransformUniform, config::Config, input_manager::InputManager,
    output_manager::OutputManager, utils::window::WindowState,
};
use neothesia_core::render::{QuadRendererFactory, TextRendererFactory};
use wgpu_jumpstart::{Gpu, Uniform};

use winit::window::Window;

#[derive(Clone)]
pub struct EventLoopProxy {
    proxy: winit::event_loop::EventLoopProxy,
    tx: std::sync::mpsc::Sender<NeothesiaEvent>,
}

impl EventLoopProxy {
    pub fn new(
        proxy: winit::event_loop::EventLoopProxy,
    ) -> (Self, std::sync::mpsc::Receiver<NeothesiaEvent>) {
        let (tx, rx) = std::sync::mpsc::channel();

        (Self { proxy, tx }, rx)
    }

    pub fn send_event(&self, event: NeothesiaEvent) -> Result<(), ()> {
        if let Err(err) = self.tx.send(event) {
            log::error!("winit event send: {err}");
            // TODO: Drop this once winit 0.31 is stable and merge conflits are no longer a concern
            return Err(());
        }
        self.proxy.wake_up();
        Ok(())
    }
}

pub struct Context {
    pub window: Arc<dyn Window>,

    pub window_state: WindowState,
    pub gpu: Gpu,

    pub transform: Uniform<TransformUniform>,
    pub text_renderer_factory: TextRendererFactory,
    pub quad_renderer_facotry: QuadRendererFactory,

    pub output_manager: OutputManager,
    pub input_manager: InputManager,
    pub config: Config,

    pub proxy: EventLoopProxy,

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
        window: Arc<dyn Window>,
        window_state: WindowState,
        proxy: EventLoopProxy,
        gpu: Gpu,
    ) -> Self {
        let transform_uniform = Uniform::new(
            &gpu.device,
            TransformUniform::default(),
            wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
        );

        let config = Config::new();

        let text_renderer_factory = TextRendererFactory::new(&gpu);
        let quad_renderer_facotry = QuadRendererFactory::new(&gpu, &transform_uniform);

        Self {
            window,

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
    }
}
