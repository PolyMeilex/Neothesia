use crate::main_state::MainState;
use crate::ui::{IcedManager, TextRenderer};
use crate::wgpu_jumpstart::{Gpu, Uniform, Window};
use crate::TransformUniform;

pub struct Target {
    pub state: MainState,
    pub window: Window,
    pub gpu: Gpu,
    pub transform_uniform: Uniform<TransformUniform>,

    pub text_renderer: TextRenderer,
    pub iced_manager: IcedManager,
}

impl Target {
    pub fn new(window: Window, gpu: Gpu) -> Self {
        let transform_uniform = Uniform::new(
            &gpu.device,
            TransformUniform::default(),
            wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
        );

        let text_renderer = TextRenderer::new(&gpu);

        let iced_manager = IcedManager::new(&gpu.device, &window);

        Self {
            state: MainState::new(),
            window,
            gpu,
            transform_uniform,

            text_renderer,
            iced_manager,
        }
    }

    pub fn resize(&mut self) {
        {
            let winit::dpi::LogicalSize { width, height } = self.window.state.logical_size;
            self.transform_uniform.data.update(width, height);
            self.transform_uniform
                .update(&mut self.gpu.encoder, &self.gpu.device);
        }

        {
            let physical_size = self.window.state.physical_size;
            self.iced_manager.viewport = iced_wgpu::Viewport::with_physical_size(
                iced_native::Size::new(physical_size.width, physical_size.height),
                self.window.state.scale_factor,
            );
        }
    }
}
