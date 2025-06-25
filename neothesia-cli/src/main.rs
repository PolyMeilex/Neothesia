use std::{default::Default, time::Duration};

use neothesia_core::{
    config::Config,
    piano_layout,
    render::{GuidelineRenderer, KeyboardRenderer, QuadPipeline, TextRenderer, WaterfallRenderer},
};
use wgpu_jumpstart::{wgpu, Gpu, TransformUniform, Uniform};

struct Recorder {
    gpu: Gpu,
    transform_uniform: Uniform<TransformUniform>,

    playback: midi_file::PlaybackState,

    quad_pipeline: QuadPipeline,
    keyboard: KeyboardRenderer,
    waterfall: WaterfallRenderer,
    text: TextRenderer,
    guidelines: GuidelineRenderer,

    config: Config,
    width: u32,
    height: u32,
}

fn get_layout(
    width: f32,
    height: f32,
    range: piano_layout::KeyboardRange,
) -> piano_layout::KeyboardLayout {
    let white_count = range.white_count();
    let neutral_width = width / white_count as f32;
    let neutral_height = height * 0.2;

    piano_layout::KeyboardLayout::from_range(
        piano_layout::Sizing::new(neutral_width, neutral_height),
        range,
    )
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

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::from_env_or_default());
        let gpu = futures::executor::block_on(Gpu::new(&instance, None)).unwrap_or_else(|err| {
            eprintln!("Failed to initialize GPU: {}", err);
            std::process::exit(1);
        });
        let args: Vec<String> = std::env::args().collect();

        let midi = if args.len() > 1 {
            midi_file::MidiFile::new(&args[1]).unwrap_or_else(|err| {
                eprintln!("Error loading MIDI file: {}", err);
                std::process::exit(1);
            })
        } else {
            eprintln!("No MIDI file provided.");
            eprintln!("Usage: neothesia-cli <midi-file>");
            std::process::exit(1);
        };

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

        let mut quad_pipeline = QuadPipeline::new(&gpu, &transform_uniform);
        quad_pipeline.init_layer(&gpu, 30); // BG
        quad_pipeline.init_layer(&gpu, 150); // FG

        let keyboard_layout = get_layout(
            width as f32,
            height as f32,
            piano_layout::KeyboardRange::new(config.piano_range()),
        );

        let mut keyboard = KeyboardRenderer::new(keyboard_layout.clone());
        keyboard.position_on_bottom_of_parent(height as f32);

        let guidelines = GuidelineRenderer::new(
            keyboard.layout().clone(),
            *keyboard.pos(),
            config.vertical_guidelines(),
            config.horizontal_guidelines(),
            midi.measures.clone(),
        );

        let mut waterfall = WaterfallRenderer::new(
            &gpu,
            &midi.tracks,
            &[],
            &config,
            &transform_uniform,
            keyboard_layout,
        );

        let playback = midi_file::PlaybackState::new(Duration::from_secs(3), midi.tracks.clone());

        waterfall.update(&gpu.queue, time_without_lead_in(&playback));

        let text = TextRenderer::new(&gpu);

        Self {
            gpu,
            transform_uniform,

            playback,

            quad_pipeline,
            keyboard,
            waterfall,
            text,
            guidelines,

            config,
            width,
            height,
        }
    }

    fn update(&mut self, delta: Duration) {
        let events = self.playback.update(delta);
        file_midi_events(&mut self.keyboard, &self.config, &events);

        let time = time_without_lead_in(&self.playback);

        self.quad_pipeline.clear();

        self.guidelines.update(
            &mut self.quad_pipeline,
            0,
            self.config.animation_speed(),
            time,
        );

        self.waterfall.update(&self.gpu.queue, time);

        self.keyboard
            .update(&mut self.quad_pipeline, 1, &mut self.text);
        self.quad_pipeline
            .prepare(&self.gpu.device, &self.gpu.queue);

        self.text.update((self.width, self.height), &mut self.gpu);
    }

    fn render(
        &mut self,
        texture: &wgpu::Texture,
        view: &wgpu::TextureView,
        texture_desc: &wgpu::TextureDescriptor<'_>,
        output_buffer: &wgpu::Buffer,
    ) {
        let bg_color = self.config.background_color();
        let bg_color = wgpu_jumpstart::Color::from(bg_color).into_linear_wgpu_color();

        {
            let mut rpass = self
                .gpu
                .encoder
                .begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: None,
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(bg_color),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });

            self.quad_pipeline
                .render(0, &self.transform_uniform, &mut rpass);
            self.waterfall.render(&self.transform_uniform, &mut rpass);
            self.quad_pipeline
                .render(1, &self.transform_uniform, &mut rpass);
            self.text.render(&mut rpass);
        }

        {
            let u32_size = std::mem::size_of::<u32>() as u32;

            self.gpu.encoder.copy_texture_to_buffer(
                wgpu::TexelCopyTextureInfo {
                    texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: Default::default(),
                },
                wgpu::TexelCopyBufferInfo {
                    buffer: output_buffer,
                    layout: wgpu::TexelCopyBufferLayout {
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
        usage: None,
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
        Some(0.0),
        Some("medium"),
    );

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

            slice.map_async(wgpu::MapMode::Read, move |_| {});

            recorder.gpu.device.poll(wgpu::PollType::Wait).unwrap();

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

        let range_start = keyboard.range().start() as usize;
        if keyboard.range().contains(key) && e.channel != 9 {
            let id = key as usize - range_start;
            let key = &mut keyboard.key_states_mut()[id];

            if is_on {
                let color = &config.color_schema()[e.track_color_id % config.color_schema().len()];
                key.pressed_by_file_on(color);
            } else {
                key.pressed_by_file_off();
            }

            keyboard.invalidate_cache();
        }
    }
}
