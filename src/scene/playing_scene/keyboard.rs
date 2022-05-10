use crate::quad_pipeline::{QuadInstance, QuadPipeline};
use crate::target::Target;
use crate::TransformUniform;
use crate::Uniform;
use lib_midi::MidiEvent;

use piano_math::range::KeyboardRange;

mod key;
pub use key::Key;
use wgpu_glyph::Section;

pub struct PianoKeyboard {
    pub quad_pipeline: QuadPipeline,
    pub keys: Vec<Key>,

    range: KeyboardRange,
}

impl PianoKeyboard {
    pub fn new(target: &mut Target) -> Self {
        let range = KeyboardRange::standard_88_keys();

        let mut quad_pipeline = QuadPipeline::new(&target.gpu, &target.transform_uniform);

        let mut keys = Vec::new();

        // 0 is reserved fo keyboard background, so it starts from 1
        let first_instance_id = 1;

        {
            let mut white_key_id: usize = 0;
            let mut black_key_id: usize = 0;

            for id in range.iter() {
                if id.is_black() {
                    keys.push(Key::new(
                        first_instance_id + range.white_count() + black_key_id,
                        true,
                    ));
                    black_key_id += 1;
                } else {
                    keys.push(Key::new(first_instance_id + white_key_id, false));
                    white_key_id += 1;
                }
            }
        }

        quad_pipeline.update_instance_buffer(
            &target.gpu.queue,
            // BG + keys
            vec![QuadInstance::default(); 1 + keys.len()],
        );

        let mut piano_keyboard = Self {
            quad_pipeline,
            keys,

            range,
        };

        piano_keyboard.resize(target).ok();

        piano_keyboard
    }

    pub fn resize(&mut self, target: &mut Target) -> Result<(), String> {
        let (window_w, window_h) = {
            let winit::dpi::LogicalSize { width, height } = target.window.state.logical_size;
            (width, height)
        };

        let neutral_width = window_w / self.range.white_count() as f32;
        let neutral_height = window_h / 5.0;

        let keyboard = piano_math::standard_88_keys(neutral_width, neutral_height);

        let y = window_h - keyboard.neutral_height as f32;

        self.quad_pipeline
            .instances_mut(&target.gpu.queue, |instances: &mut Vec<QuadInstance>| {
                // Keyboard background
                instances[0] = QuadInstance {
                    position: [0.0, window_h - keyboard.neutral_height],
                    size: [window_w, keyboard.neutral_height],
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

                    instances[self.keys[id].instance_id] = QuadInstance::from(&self.keys[id]);
                }
            });

        Ok(())
    }

    pub fn update(&mut self, target: &mut Target) {
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

    pub fn update_note_events(&mut self, target: &mut Target, events: &[MidiEvent]) {
        if events.is_empty() {
            return;
        }

        let keys = &mut self.keys;
        let color_schema = &target.config.color_schema;
        let range = &self.range;

        let updater = |instances: &mut Vec<QuadInstance>| {
            for e in events {
                match e.message {
                    lib_midi::midly::MidiMessage::NoteOn { key, .. } => {
                        let key = key.as_int();

                        if range.contains(key) && e.channel != 9 {
                            let id = key as usize - 21;
                            let key = &mut keys[id];

                            let color = &color_schema[e.track_id % color_schema.len()];
                            key.set_color(color);

                            instances[key.instance_id] = QuadInstance::from(&*key);
                        }
                    }
                    lib_midi::midly::MidiMessage::NoteOff { key, .. } => {
                        let key = key.as_int();
                        if range.contains(key) && e.channel != 9 {
                            let id = key as usize - 21;
                            let key = &mut keys[id];

                            key.reset_color();

                            instances[key.instance_id] = QuadInstance::from(&*key);
                        }
                    }
                    _ => continue,
                };
            }
        };

        self.quad_pipeline.instances_mut(&target.gpu.queue, updater);
    }

    pub fn reset_notes(&mut self, target: &mut Target) {
        let keys = &mut self.keys;
        let updater = |instances: &mut Vec<QuadInstance>| {
            for key in keys.iter_mut() {
                key.reset_color();
                instances[key.instance_id] = QuadInstance::from(&*key);
            }
        };

        self.quad_pipeline.instances_mut(&target.gpu.queue, updater);
    }

    pub fn render<'rpass>(
        &'rpass mut self,
        transform_uniform: &'rpass Uniform<TransformUniform>,
        render_pass: &mut wgpu::RenderPass<'rpass>,
    ) {
        self.quad_pipeline.render(transform_uniform, render_pass);
    }
}
