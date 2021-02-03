use crate::{
    scene::{playing_scene::PlayingScene, Scene},
    target::Target,
    wgpu_jumpstart::{Gpu, Window},
};

pub struct Recorder {
    pub target: Target,

    pub scene: PlayingScene,
}

impl Recorder {
    pub fn new(gpu: Gpu, window: Window) -> Self {
        let mut target = Target::new(window, gpu);

        // target.resize();
        target.gpu.submit().unwrap();
        let scene = PlayingScene::new(&mut target);

        Self { target, scene }
    }

    pub fn resize(&mut self) {
        self.target.resize();
        self.scene.resize(&mut self.target);

        self.target.gpu.submit().unwrap();
    }

    pub fn update(&mut self) {
        self.scene.update(&mut self.target);
    }

    pub fn render<'a>(
        &mut self,
        texture: &wgpu::Texture,
        view: &wgpu::TextureView,
        texture_desc: &wgpu::TextureDescriptor<'a>,
        output_buffer: &wgpu::Buffer,
    ) {
        self.target
            .gpu
            .clear(view, self.target.state.config.background_color.into());

        self.scene.render(&mut self.target, view);

        self.target
            .text_renderer
            .render(&self.target.window, &mut self.target.gpu, view);

        {
            let u32_size = std::mem::size_of::<u32>() as u32;

            self.target.gpu.encoder.copy_texture_to_buffer(
                wgpu::TextureCopyView {
                    texture: &texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                },
                wgpu::BufferCopyView {
                    buffer: &output_buffer,
                    layout: wgpu::TextureDataLayout {
                        offset: 0,
                        bytes_per_row: u32_size * 1920,
                        rows_per_image: 1080,
                    },
                },
                texture_desc.size,
            );

            self.target.gpu.submit().unwrap();
        }
    }
}
