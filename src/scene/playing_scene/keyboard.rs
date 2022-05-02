use crate::midi_event::MidiEvent;
use crate::quad_pipeline::{QuadInstance, QuadPipeline};
use crate::target::Target;
use crate::TransformUniform;
use crate::Uniform;

mod range;
use range::KeyboardRange;

mod key;
pub use key::Key;

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
            &mut target.gpu.encoder,
            &target.gpu.device,
            // BG + keys
            vec![QuadInstance::default(); 1 + keys.len()],
        );

        let mut piano_keyboard = Self {
            quad_pipeline,
            keys,

            range,
        };

        piano_keyboard.resize(target);

        piano_keyboard
    }

    pub fn resize(&mut self, target: &mut Target) {
        let (window_w, window_h) = {
            let winit::dpi::LogicalSize { width, height } = target.window.state.logical_size;
            (width, height)
        };

        let white_width = window_w / self.range.white_count() as f32;
        let white_height = window_h / 5.0;

        let mut white_key_id: usize = 0;

        let keys = &mut self.keys;

        let updater = |instances: &mut Vec<QuadInstance>| {
            // Keyboard background
            instances[0] = QuadInstance {
                position: [0.0, window_h - white_height],
                size: [window_w, white_height],
                ..Default::default()
            };

            for key in keys.iter_mut() {
                let x = white_key_id as f32 * white_width;
                let y = window_h - white_height;

                if key.is_black() {
                    let black_width = white_width / 1.5;
                    let black_height = white_height / 1.5;

                    key.pos = (x - black_width / 2.0, y);
                    key.size = (black_width, black_height);
                    instances[key.instance_id] = QuadInstance::from(&*key);
                } else {
                    key.pos = (x, y);
                    key.size = (white_width - 1.0, white_height);
                    instances[key.instance_id] = QuadInstance::from(&*key);
                    white_key_id += 1;
                }
            }
        };

        self.quad_pipeline
            .instances_mut(&mut target.gpu.encoder, &target.gpu.device, updater);
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
                match *e {
                    MidiEvent::NoteOn {
                        key,
                        channel,
                        track_id,
                        ..
                    } => {
                        if range.contains(key) && channel != 9 {
                            let id = key as usize - 21;
                            let key = &mut keys[id];

                            let color = &color_schema[track_id as usize % color_schema.len()];
                            key.set_color(color);

                            instances[key.instance_id] = QuadInstance::from(&*key);
                        }
                    }
                    MidiEvent::NoteOff { key, channel } => {
                        if range.contains(key) && channel != 9 {
                            let id = key as usize - 21;
                            let key = &mut keys[id];

                            key.reset_color();

                            instances[key.instance_id] = QuadInstance::from(&*key);
                        }
                    }
                }
            }
        };

        self.quad_pipeline
            .instances_mut(&mut target.gpu.encoder, &target.gpu.device, updater);
    }

    pub fn reset_notes(&mut self, target: &mut Target) {
        let keys = &mut self.keys;
        let updater = |instances: &mut Vec<QuadInstance>| {
            for key in keys.iter_mut() {
                key.reset_color();
                instances[key.instance_id] = QuadInstance::from(&*key);
            }
        };

        self.quad_pipeline
            .instances_mut(&mut target.gpu.encoder, &target.gpu.device, updater);
    }

    pub fn render<'rpass>(
        &'rpass mut self,
        transform_uniform: &'rpass Uniform<TransformUniform>,
        render_pass: &mut wgpu::RenderPass<'rpass>,
    ) {
        self.quad_pipeline.render(transform_uniform, render_pass);
    }
}
