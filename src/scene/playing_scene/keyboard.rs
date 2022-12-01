use crate::config::Config;
use crate::target::Target;
use crate::TransformUniform;
use crate::Uniform;
use lib_midi::MidiEvent;
use neothesia_pipelines::quad::{QuadInstance, QuadPipeline};

use piano_math::range::KeyboardRange;

mod key;
pub use key::Key;
use wgpu_glyph::Section;

pub struct PianoKeyboard {
    pub quad_pipeline: QuadPipeline,
    pub keys: Vec<Key>,
    black_background: QuadInstance,

    range: KeyboardRange,
    window_size: winit::dpi::LogicalSize<f32>,
    should_reupload: bool,
}

impl PianoKeyboard {
    pub fn new(target: &mut Target) -> Self {
        let range = KeyboardRange::standard_88_keys();

        let quad_pipeline = QuadPipeline::new(&target.gpu, &target.transform_uniform);
        let keys: Vec<Key> = range.iter().map(|id| Key::new(id.is_black())).collect();

        let mut piano_keyboard = Self {
            quad_pipeline,
            keys,
            black_background: QuadInstance::default(),

            range,
            window_size: target.window.state.logical_size,
            should_reupload: false,
        };

        piano_keyboard.calculate_positions();
        piano_keyboard
    }

    /// Calculate positions of keys
    fn calculate_positions(&mut self) {
        let neutral_width = self.window_size.width / self.range.white_count() as f32;
        let neutral_height = self.window_size.height / 5.0;

        let keyboard = piano_math::standard_88_keys(neutral_width, neutral_height);

        let y = self.window_size.height - keyboard.neutral_height as f32;

        self.black_background = QuadInstance {
            position: [0.0, self.window_size.height - keyboard.neutral_height],
            size: [self.window_size.width, keyboard.neutral_height],
            color: [0.0, 0.0, 0.0, 1.0],
            ..Default::default()
        };

        for (id, key) in keyboard.keys.iter().enumerate() {
            self.keys[id].pos = (key.x(), y);
            self.keys[id].note_id = key.note_id();

            match key.kind() {
                piano_math::KeyKind::Neutral => {
                    self.keys[id].size = (key.width() - 1.0, key.height());
                }
                piano_math::KeyKind::Sharp => {
                    self.keys[id].size = (key.width(), key.height());
                }
            }
        }

        self.should_reupload = true;
    }

    pub fn resize(&mut self, window_size: winit::dpi::LogicalSize<f32>) {
        self.window_size = window_size;
        self.calculate_positions();
    }

    pub fn update_note_events(&mut self, config: &Config, events: &[MidiEvent]) {
        for e in events {
            match e.message {
                lib_midi::midly::MidiMessage::NoteOn { key, .. } => {
                    let key = key.as_int();

                    if self.range.contains(key) && e.channel != 9 {
                        let id = key as usize - 21;
                        let key = &mut self.keys[id];

                        let color = &config.color_schema[e.track_id % config.color_schema.len()];
                        key.set_color(color);
                        self.should_reupload = true;
                    }
                }
                lib_midi::midly::MidiMessage::NoteOff { key, .. } => {
                    let key = key.as_int();
                    if self.range.contains(key) && e.channel != 9 {
                        let id = key as usize - 21;
                        let key = &mut self.keys[id];

                        key.reset_color();
                        self.should_reupload = true;
                    }
                }
                _ => continue,
            };
        }
    }

    pub fn reset_notes(&mut self) {
        for key in self.keys.iter_mut() {
            key.reset_color();
        }
        self.should_reupload = true;
    }

    /// Reupload instances to GPU
    fn reupload(&mut self, queue: &wgpu::Queue) {
        self.quad_pipeline.with_instances_mut(queue, |instances| {
            instances.clear();
            instances.push(self.black_background);

            for key in self.keys.iter().filter(|key| !key.is_black()) {
                instances.push(QuadInstance::from(key));
            }

            for key in self.keys.iter().filter(|key| key.is_black()) {
                instances.push(QuadInstance::from(key));
            }
        });
        self.should_reupload = false;
    }

    pub fn update(&mut self, target: &mut Target) {
        if self.should_reupload {
            self.reupload(&target.gpu.queue);
        }

        for (id, key) in self.keys.iter().filter(|key| key.note_id == 0).enumerate() {
            let (x, y) = key.pos;
            let (w, h) = key.size;

            let size = w * 0.7;

            target.text_renderer.queue_text(Section {
                screen_position: (x + w / 2.0, y + h - size * 1.3),
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
