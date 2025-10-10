use std::{default::Default, time::Duration};

use neothesia_core::{
    config::Config,
    piano_layout,
    render::{
        GuidelineRenderer, KeyboardRenderer, QuadRenderer, QuadRendererFactory, TextRenderer,
        TextRendererFactory, WaterfallRenderer,
    },
};
use wgpu_jumpstart::{Gpu, TransformUniform, Uniform, wgpu};

mod cli;

struct Recorder {
    gpu: Gpu,

    playback: midi_file::PlaybackState,

    quad_renderer_bg: QuadRenderer,
    quad_renderer_fg: QuadRenderer,
    keyboard: KeyboardRenderer,
    waterfall: WaterfallRenderer,
    text: TextRenderer,
    guidelines: GuidelineRenderer,

    config: Config,
    width: u32,
    height: u32,

    synth: oxisynth::Synth,
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
    fn new(args: &cli::Args) -> Self {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::from_env_or_default());
        let gpu = pollster::block_on(Gpu::new(&instance, None)).unwrap_or_else(|err| {
            eprintln!("Failed to initialize GPU: {err}");
            std::process::exit(1);
        });

        let midi = midi_file::MidiFile::new(&args.midi).unwrap_or_else(|err| {
            eprintln!("Error loading MIDI file: {err}");
            std::process::exit(1);
        });

        let config = Config::new();

        let width = args.width;
        let height = args.height;

        let mut transform_uniform = TransformUniform::default();
        transform_uniform.update(width as f32, height as f32, 1.0);

        let transform_uniform = Uniform::new(
            &gpu.device,
            transform_uniform,
            wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
        );

        let quad_renderer_factory = QuadRendererFactory::new(&gpu, &transform_uniform);

        let quad_renderer_bg = quad_renderer_factory.new_renderer();
        let quad_renderer_fg = quad_renderer_factory.new_renderer();

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

        waterfall.update(time_without_lead_in(&playback));

        let text = TextRendererFactory::new(&gpu);
        let text = text.new_renderer();

        let mut synth = oxisynth::Synth::new(oxisynth::SynthDescriptor {
            sample_rate: 44100.0,
            gain: 0.5,
            ..Default::default()
        })
        .unwrap();

        if let Some(sf2) = args.soundfont.as_ref() {
            let mut file = std::fs::File::open(sf2).unwrap();
            let font = oxisynth::SoundFont::load(&mut file).unwrap();
            synth.add_font(font, true);
        }

        Self {
            gpu,

            playback,

            quad_renderer_bg,
            quad_renderer_fg,
            keyboard,
            waterfall,
            text,
            guidelines,

            config,
            width,
            height,

            synth,
        }
    }

    fn update(&mut self, delta: Duration) {
        let events = self.playback.update(delta);
        file_midi_events(&mut self.synth, &mut self.keyboard, &self.config, &events);

        let time = time_without_lead_in(&self.playback);

        self.quad_renderer_bg.clear();
        self.quad_renderer_fg.clear();

        self.guidelines.update(
            &mut self.quad_renderer_bg,
            self.config.animation_speed(),
            1.0,
            time,
            neothesia_core::dpi::LogicalSize::new(self.width as f32, self.height as f32),
        );

        self.waterfall.update(time);

        self.keyboard
            .update(&mut self.quad_renderer_fg, &mut self.text);

        self.quad_renderer_bg.prepare();
        self.quad_renderer_fg.prepare();

        self.text.update(
            neothesia_core::dpi::PhysicalSize::new(self.width, self.height),
            1.0,
        );
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
            let rpass = self
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
                        depth_slice: None,
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });
            let mut rpass = wgpu_jumpstart::RenderPass::new(rpass, texture.size());

            self.quad_renderer_bg.render(&mut rpass);
            self.waterfall.render(&mut rpass);
            self.quad_renderer_fg.render(&mut rpass);
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
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("neothesia=info"))
        .init();

    let args = cli::Args::get();

    let mut recorder = Recorder::new(&args);

    let texture_desc = wgpu::TextureDescriptor {
        size: wgpu::Extent3d {
            width: recorder.width,
            height: recorder.height,
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

    let (encoder_info, mut encoder) =
        ffmpeg_encoder::new(&args.out, recorder.width, recorder.height);

    let frame_size = encoder_info.frame_size;

    let start = std::time::Instant::now();

    let frame_time = Duration::from_secs(1) / 60;
    const SAMPLE_TIME: usize = 44100 / 60;

    let mut audio_buffer_l: Vec<f32> = Vec::with_capacity(frame_size);
    let mut audio_buffer_r: Vec<f32> = Vec::with_capacity(frame_size);

    println!("Encoding started:");
    let mut n = 1;
    while recorder.playback.percentage() * 100.0 < 101.0 {
        let output_buffer = recorder.gpu.device.create_buffer(&output_buffer_desc);

        recorder.update(frame_time);
        recorder.render(&texture, view, &texture_desc, &output_buffer);

        for _ in 0..SAMPLE_TIME {
            let val = recorder.synth.read_next();
            audio_buffer_l.push(val.0);
            audio_buffer_r.push(val.0);
        }

        if audio_buffer_l.len() >= frame_size {
            encoder(ffmpeg_encoder::Frame::Audio(
                &audio_buffer_l[..frame_size],
                &audio_buffer_r[..frame_size],
            ));
            audio_buffer_l.drain(..frame_size);
            audio_buffer_r.drain(..frame_size);
        }

        {
            let slice = output_buffer.slice(..);

            slice.map_async(wgpu::MapMode::Read, move |_| {});

            recorder
                .gpu
                .device
                .poll(wgpu::PollType::Wait {
                    submission_index: None,
                    timeout: None,
                })
                .unwrap();

            let mapping = slice.get_mapped_range();

            let data: &[u8] = &mapping;

            encoder(ffmpeg_encoder::Frame::Vide(data));

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

    for (l, r) in audio_buffer_l
        .chunks(frame_size)
        .zip(audio_buffer_r.chunks(frame_size))
    {
        encoder(ffmpeg_encoder::Frame::Audio(l, r));
    }

    encoder(ffmpeg_encoder::Frame::Terminator);
}

fn file_midi_events(
    synth: &mut oxisynth::Synth,
    keyboard: &mut KeyboardRenderer,
    config: &Config,
    events: &[&midi_file::MidiEvent],
) {
    use midi_file::midly::MidiMessage;

    for e in events {
        let (is_on, key, vel) = match e.message {
            MidiMessage::NoteOn { key, vel, .. } => (true, key.as_int(), vel),
            MidiMessage::NoteOff { key, .. } => (false, key.as_int(), 0.into()),
            _ => continue,
        };

        if is_on {
            synth
                .send_event(oxisynth::MidiEvent::NoteOn {
                    channel: 1,
                    key,
                    vel: vel.as_int(),
                })
                .ok();
        } else {
            synth
                .send_event(oxisynth::MidiEvent::NoteOff { channel: 1, key })
                .ok();
        }

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
