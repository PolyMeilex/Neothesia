use std::{default::Default, time::Duration};

use neothesia_core::{
    config::Config,
    render::{KeyboardRenderer, TextRenderer, WaterfallRenderer},
};
use wgpu_jumpstart::{wgpu, Gpu, TransformUniform, Uniform};

struct Recorder {
    gpu: Gpu,
    transform_uniform: Uniform<TransformUniform>,

    playback: midi_file::PlaybackState,
    midi: midi_file::Midi,

    keyboard: KeyboardRenderer,
    waterfall: WaterfallRenderer,
    text: TextRenderer,

    config: Config,
    width: u32,
    height: u32,
}

fn get_layout(width: f32, height: f32) -> piano_math::KeyboardLayout {
    let range = piano_math::KeyboardRange::standard_88_keys();
    let white_count = range.white_count();
    let neutral_width = width / white_count as f32;
    let neutral_height = height * 0.2;

    piano_math::KeyboardLayout::from_range(neutral_width, neutral_height, range)
}

fn time_without_lead_in(playback: &midi_file::PlaybackState) -> f32 {
    playback.time().as_secs_f32() - playback.leed_in().as_secs_f32()
}

impl Recorder {
    fn new() -> Self {
        env_logger::Builder::from_env(
            env_logger::Env::default().default_filter_or("neothesia=info"),
        )
        .init();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu_jumpstart::default_backends(),
            ..Default::default()
        });
        let gpu = futures::executor::block_on(Gpu::new(&instance, None)).unwrap();

        let args: Vec<String> = std::env::args().collect();

        let midi = if args.len() > 1 {
            midi_file::Midi::new(&args[1]).ok()
        } else {
            None
        }
        .unwrap();

        let config = Config::new();

        let width = 1920;
        let height = 1080;

        let mut transform_uniform = TransformUniform::default();
        transform_uniform.update(width as f32, height as f32, 1.0);

        let transform_uniform = Uniform::new(
            &gpu.device,
            transform_uniform,
            wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
        );

        let keyboard_layout = get_layout(width as f32, height as f32);

        let mut keyboard = KeyboardRenderer::new(&gpu, &transform_uniform, keyboard_layout.clone());

        keyboard.position_on_bottom_of_parent(height as f32);

        let mut waterfall =
            WaterfallRenderer::new(&gpu, &midi, &config, &transform_uniform, keyboard_layout);

        let playback = midi_file::PlaybackState::new(Duration::from_secs(3), &midi.merged_track);

        waterfall.update(&gpu.queue, time_without_lead_in(&playback));

        let text = TextRenderer::new(&gpu);

        Self {
            gpu,
            transform_uniform,

            playback,
            midi,

            keyboard,
            waterfall,
            text,

            config,
            width,
            height,
        }
    }

    fn update(&mut self, delta: Duration) {
        let events = self.playback.update(&self.midi.merged_track, delta);
        file_midi_events(&mut self.keyboard, &self.config, &events);

        self.waterfall
            .update(&self.gpu.queue, time_without_lead_in(&self.playback));

        self.keyboard
            .update(&self.gpu.queue, self.text.glyph_brush());
    }

    fn render(
        &mut self,
        texture: &wgpu::Texture,
        view: &wgpu::TextureView,
        texture_desc: &wgpu::TextureDescriptor<'_>,
        output_buffer: &wgpu::Buffer,
    ) {
        self.gpu.clear(view, self.config.background_color.into());

        {
            let mut render_pass = self
                .gpu
                .encoder
                .begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: None,
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: true,
                        },
                    })],
                    depth_stencil_attachment: None,
                });

            self.waterfall
                .render(&self.transform_uniform, &mut render_pass);

            self.keyboard
                .render(&self.transform_uniform, &mut render_pass);
        }

        self.text
            .render((self.width as f32, self.height as f32), &mut self.gpu, view);

        {
            let u32_size = std::mem::size_of::<u32>() as u32;

            self.gpu.encoder.copy_texture_to_buffer(
                wgpu::ImageCopyTexture {
                    texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: Default::default(),
                },
                wgpu::ImageCopyBuffer {
                    buffer: output_buffer,
                    layout: wgpu::ImageDataLayout {
                        offset: 0,
                        bytes_per_row: Some(u32_size * self.width),
                        rows_per_image: Some(self.height),
                    },
                },
                texture_desc.size,
            );

            self.gpu.submit();
        }
    }
}

fn main() {
    let mut recorder = Recorder::new();

    let texture_desc = wgpu::TextureDescriptor {
        size: wgpu::Extent3d {
            width: 1920,
            height: 1080,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Bgra8UnormSrgb,
        view_formats: &[wgpu::TextureFormat::Bgra8UnormSrgb],
        usage: wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::RENDER_ATTACHMENT,
        label: None,
    };
    let texture = recorder.gpu.device.create_texture(&texture_desc);
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
    let output_buffer_size = (u32_size * recorder.width * recorder.height) as wgpu::BufferAddress;

    let output_buffer_desc = wgpu::BufferDescriptor {
        size: output_buffer_size,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        label: None,
        mapped_at_creation: false,
    };

    std::fs::create_dir("./out").ok();
    let mut encoder = mpeg_encoder::Encoder::new(
        "./out/video.mp4",
        recorder.width as usize,
        recorder.height as usize,
    );

    encoder.init(Some(0.0), Some("medium"));

    let start = std::time::Instant::now();

    println!("Encoding started:");
    let mut n = 1;
    while recorder.playback.percentage() * 100.0 < 101.0 {
        let output_buffer = recorder.gpu.device.create_buffer(&output_buffer_desc);

        let frame_time = Duration::from_secs(1) / 60;
        recorder.update(frame_time);
        recorder.render(&texture, view, &texture_desc, &output_buffer);

        {
            let slice = output_buffer.slice(..);
            futures::executor::block_on(async {
                let (tx, rx) = futures::channel::oneshot::channel();

                slice.map_async(wgpu::MapMode::Read, move |_| {
                    tx.send(()).unwrap();
                });

                recorder.gpu.device.poll(wgpu::Maintain::Wait);

                rx.await.unwrap();

                let mapping = slice.get_mapped_range();

                let data: &[u8] = &mapping;
                encoder.encode_bgra(1920, 1080, data, false);
                print!(
                    "\r Encoded {} frames ({}s, {}%) in {}s",
                    n,
                    (n as f32 / 60.0).round(),
                    (recorder.playback.percentage() * 100.0).round().min(100.0),
                    start.elapsed().as_secs()
                );
            });
        }

        n += 1;
    }
}

fn file_midi_events(
    keyboard: &mut KeyboardRenderer,
    config: &Config,
    events: &[&midi_file::MidiEvent],
) {
    use midi_file::midly::MidiMessage;

    for e in events {
        let (is_on, key) = match e.message {
            MidiMessage::NoteOn { key, .. } => (true, key.as_int()),
            MidiMessage::NoteOff { key, .. } => (false, key.as_int()),
            _ => continue,
        };

        if keyboard.range().contains(key) && e.channel != 9 {
            let id = key as usize - 21;
            let key = &mut keyboard.key_states_mut()[id];

            if is_on {
                let color = &config.color_schema[e.track_color_id % config.color_schema.len()];
                key.pressed_by_file_on(color);
            } else {
                key.pressed_by_file_off();
            }

            keyboard.queue_reupload();
        }
    }
}
