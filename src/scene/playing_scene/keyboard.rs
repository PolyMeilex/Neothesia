use crate::{
    config::Config,
    utils::{Point, Size},
    TransformUniform, Uniform,
};

use neothesia_pipelines::quad::{QuadInstance, QuadPipeline};
use piano_math::range::KeyboardRange;
use wgpu_glyph::{GlyphBrush, Section};

mod key;
pub use key::Key;
use wgpu_jumpstart::Gpu;

pub struct PianoKeyboard {
    pos: Point<f32>,
    size: Size<f32>,

    keys: Vec<Key>,
    range: KeyboardRange,

    quad_pipeline: QuadPipeline,
    should_reupload: bool,
}

impl PianoKeyboard {
    pub fn new(
        gpu: &Gpu,
        transform_uniform: &Uniform<TransformUniform>,
        window_size: winit::dpi::LogicalSize<f32>,
    ) -> Self {
        let range = KeyboardRange::standard_88_keys();

        let quad_pipeline = QuadPipeline::new(gpu, transform_uniform);
        let keys: Vec<Key> = range.iter().map(|id| Key::new(id.is_black())).collect();

        let mut piano_keyboard = Self {
            pos: Default::default(),
            size: Default::default(),

            keys,
            range,

            quad_pipeline,
            should_reupload: false,
        };

        piano_keyboard.resize(window_size);
        piano_keyboard
    }

    pub fn keys(&self) -> &[Key] {
        &self.keys
    }

    /// Calculate positions of keys
    fn calculate_positions(&mut self) {
        let neutral_width = self.size.w / self.range.white_count() as f32;
        let keyboard = piano_math::standard_88_keys(neutral_width, self.size.h);

        for (id, key) in keyboard.keys.iter().enumerate() {
            self.keys[id].note_id = key.note_id();

            self.keys[id].pos = self.pos;
            self.keys[id].pos.x += key.x();

            self.keys[id].size = key.size().into();

            if let piano_math::KeyKind::Neutral = key.kind() {
                self.keys[id].size.w -= 1.0;
            }
        }

        self.queue_reupload();
    }

    pub fn resize(&mut self, window_size: winit::dpi::LogicalSize<f32>) {
        self.size.w = window_size.width;
        self.size.h = window_size.height * 0.2;

        self.pos.x = 0.0;
        self.pos.y = window_size.height - self.size.h;

        self.calculate_positions();
    }

    pub fn user_midi_event(&mut self, event: &crate::MidiEvent) {
        match event {
            crate::MidiEvent::NoteOn { key, .. } => {
                if self.range.contains(*key) {
                    let id = *key as usize - 21;
                    let key = &mut self.keys[id];

                    key.set_pressed_by_user(true);
                    self.queue_reupload();
                }
            }
            crate::MidiEvent::NoteOff { key, .. } => {
                if self.range.contains(*key) {
                    let id = *key as usize - 21;
                    let key = &mut self.keys[id];

                    key.set_pressed_by_user(false);
                    self.queue_reupload();
                }
            }
        }
    }

    pub fn file_midi_events(&mut self, config: &Config, events: &[lib_midi::MidiEvent]) {
        for e in events {
            match e.message {
                lib_midi::midly::MidiMessage::NoteOn { key, .. } => {
                    let key = key.as_int();

                    if self.range.contains(key) && e.channel != 9 {
                        let id = key as usize - 21;
                        let key = &mut self.keys[id];

                        let color = &config.color_schema[e.track_id % config.color_schema.len()];
                        key.pressed_by_file_on(color);
                        self.queue_reupload();
                    }
                }
                lib_midi::midly::MidiMessage::NoteOff { key, .. } => {
                    let key = key.as_int();
                    if self.range.contains(key) && e.channel != 9 {
                        let id = key as usize - 21;
                        let key = &mut self.keys[id];

                        key.pressed_by_file_off();
                        self.queue_reupload();
                    }
                }
                _ => continue,
            };
        }
    }

    pub fn reset_notes(&mut self) {
        for key in self.keys.iter_mut() {
            key.pressed_by_file_off();
        }
        self.queue_reupload();
    }

    fn queue_reupload(&mut self) {
        self.should_reupload = true;
    }

    /// Reupload instances to GPU
    fn reupload(&mut self, queue: &wgpu::Queue) {
        self.quad_pipeline.with_instances_mut(queue, |instances| {
            instances.clear();

            // black_background
            instances.push(QuadInstance {
                position: self.pos.into(),
                size: self.size.into(),
                color: [0.0, 0.0, 0.0, 1.0],
                ..Default::default()
            });

            for key in self.keys.iter().filter(|key| !key.is_black()) {
                instances.push(QuadInstance::from(key));
            }

            for key in self.keys.iter().filter(|key| key.is_black()) {
                instances.push(QuadInstance::from(key));
            }
        });
        self.should_reupload = false;
    }

    pub fn update(&mut self, queue: &wgpu::Queue, brush: &mut GlyphBrush<()>) {
        if self.should_reupload {
            self.reupload(queue);
        }

        for (id, key) in self.keys.iter().filter(|key| key.note_id == 0).enumerate() {
            let Point { x, y } = key.pos;
            let Size { w, h } = key.size;

            let size = w * 0.7;

            brush.queue(Section {
                screen_position: (x + w / 2.0, y + h - size * 1.2),
                text: vec![wgpu_glyph::Text::new(&format!("C{}", id + 1))
                    .with_color([0.6, 0.6, 0.6, 1.0])
                    .with_scale(size)],
                bounds: (w, f32::INFINITY),
                layout: wgpu_glyph::Layout::default()
                    .h_align(wgpu_glyph::HorizontalAlign::Center)
                    .v_align(wgpu_glyph::VerticalAlign::Top),
            })
        }
    }

    pub fn render<'rpass>(
        &'rpass mut self,
        transform_uniform: &'rpass Uniform<TransformUniform>,
        render_pass: &mut wgpu::RenderPass<'rpass>,
    ) {
        self.quad_pipeline.render(transform_uniform, render_pass);
    }
}
