#![cfg(feature = "record")]

use std::{default::Default, num::NonZeroU32, time::Duration};

use neothesia::{
    scene::{playing_scene::PlayingScene, Scene},
    target::Target,
    wgpu_jumpstart,
};

pub struct Recorder {
    pub target: Target,

    pub scene: PlayingScene,
}

impl Recorder {
    pub fn new(mut target: Target) -> Self {
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

    pub fn update(&mut self, delta: Duration) {
        self.scene.update(&mut self.target, delta);
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
            .clear(view, self.target.config.background_color.into());

        self.scene.render(&mut self.target, view);

        self.target
            .text_renderer
            .render(&self.target.window, &mut self.target.gpu, view);

        {
            let u32_size = std::mem::size_of::<u32>() as u32;

            self.target.gpu.encoder.copy_texture_to_buffer(
                wgpu::ImageCopyTexture {
                    texture: &texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: Default::default(),
                },
                wgpu::ImageCopyBuffer {
                    buffer: &output_buffer,
                    layout: wgpu::ImageDataLayout {
                        offset: 0,
                        bytes_per_row: NonZeroU32::new(u32_size * 1920),
                        rows_per_image: NonZeroU32::new(1080),
                    },
                },
                texture_desc.size,
            );

            self.target.gpu.submit().unwrap();
        }
    }
}

fn main() {
    let builder = winit::window::WindowBuilder::new()
        .with_inner_size(winit::dpi::LogicalSize {
            width: 1920,
            height: 1080,
        })
        .with_visible(false);

    let (_event_loop, target) = neothesia::init(builder);
    let mut recorder = Recorder::new(target);

    recorder.resize();
    let texture_desc = wgpu::TextureDescriptor {
        size: wgpu::Extent3d {
            width: 1920,
            height: 1080,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu_jumpstart::TEXTURE_FORMAT,
        usage: wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::RENDER_ATTACHMENT,
        label: None,
    };
    let texture = recorder.target.gpu.device.create_texture(&texture_desc);
    let view = &texture.create_view(&wgpu::TextureViewDescriptor {
        label: None,
        format: None,
        dimension: None,
        aspect: wgpu::TextureAspect::All,
        base_mip_level: 0,
        mip_level_count: None,
        base_array_layer: 0,
        array_layer_count: None,
    });

    let u32_size = std::mem::size_of::<u32>() as u32;
    let output_buffer_size = (u32_size * 1920 * 1080) as wgpu::BufferAddress;

    let output_buffer_desc = wgpu::BufferDescriptor {
        size: output_buffer_size,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        label: None,
        mapped_at_creation: false,
    };

    std::fs::create_dir("./out").ok();
    let mut encoder = mpeg_encoder::Encoder::new("./out/video.mp4", 1920, 1080);

    encoder.init(Some(0.0), Some("medium"));

    let start = std::time::Instant::now();

    println!("Encoding started:");
    let mut n = 1;
    while recorder.scene.playback_progress() < 101.0 {
        let output_buffer = recorder
            .target
            .gpu
            .device
            .create_buffer(&output_buffer_desc);

        let frame_time = Duration::from_secs(1) / 60;
        recorder.update(frame_time);
        recorder.render(&texture, &view, &texture_desc, &output_buffer);

        {
            let slice = output_buffer.slice(..);
            neothesia::block_on(async {
                let task = slice.map_async(wgpu::MapMode::Read);

                recorder.target.gpu.device.poll(wgpu::Maintain::Wait);

                task.await.unwrap();

                let mapping = slice.get_mapped_range();

                let data: &[u8] = &mapping;
                encoder.encode_rgba(1920, 1080, data, false);
                print!(
                    "\r Encoded {} frames ({}s, {}%) in {}s",
                    n,
                    (n as f32 / 60.0).round(),
                    recorder.scene.playback_progress().round(),
                    start.elapsed().as_secs()
                );
            });
        }

        n += 1;
    }
}
