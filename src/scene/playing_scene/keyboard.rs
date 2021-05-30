use crate::config::ColorSchema;
use crate::quad_pipeline::{QuadInstance, QuadPipeline};
use crate::target::Target;
use crate::wgpu_jumpstart::Color;
use crate::TransformUniform;
use crate::Uniform;

// const KEY_C: u8 = 0;
const KEY_CIS: u8 = 1;
// const KEY_D: u8 = 2;
const KEY_DIS: u8 = 3;
// const KEY_E: u8 = 4;
// const KEY_F: u8 = 5;
const KEY_FIS: u8 = 6;
// const KEY_G: u8 = 7;
const KEY_GIS: u8 = 8;
// const KEY_A: u8 = 9;
const KEY_AIS: u8 = 10;
// const KEY_B: u8 = 11;

pub struct Key {
    instance_id: usize,
    pos: (f32, f32),
    size: (f32, f32),
    is_black: bool,

    color: Color,
}

impl Key {
    fn new(instance_id: usize, is_black: bool) -> Self {
        Self {
            instance_id,
            pos: (0.0, 0.0),
            size: (0.0, 0.0),
            is_black,

            color: if is_black {
                Color::new(0.0, 0.0, 0.0, 1.0)
            } else {
                Color::new(1.0, 1.0, 1.0, 1.0)
            },
        }
    }

    pub fn x_position(&self) -> f32 {
        self.pos.0
    }

    pub fn width(&self) -> f32 {
        self.size.0
    }

    pub fn is_black(&self) -> bool {
        self.is_black
    }

    pub fn set_color(&mut self, schem: &ColorSchema) {
        let (r, g, b) = if self.is_black {
            schem.dark
        } else {
            schem.base
        };
        self.color = Color::from_rgba8(r, g, b, 1.0);
    }

    pub fn reset_color(&mut self) {
        if self.is_black {
            self.color = Color::new(0.0, 0.0, 0.0, 1.0);
        } else {
            self.color = Color::new(1.0, 1.0, 1.0, 1.0);
        }
    }
}

pub struct PianoKeyboard {
    pub keyboard_pipeline: QuadPipeline,
    pub keys: Vec<Key>,

    white_key_count: usize,
    black_key_count: usize,
}

impl PianoKeyboard {
    pub fn new(target: &mut Target) -> Self {
        let white_key_count = 52;
        let mut keyboard_pipeline = QuadPipeline::new(&mut target.gpu, &target.transform_uniform);

        let mut keys = Vec::new();

        {
            let mut white_key_id: usize = 0;
            let mut black_key_id: usize = 0;

            for id in 0..88 {
                let key_id = id + 9;
                let note_id = key_id % 12;

                if note_id == KEY_CIS
                    || note_id == KEY_DIS
                    || note_id == KEY_FIS
                    || note_id == KEY_GIS
                    || note_id == KEY_AIS
                {
                    keys.push(Key::new(white_key_count + black_key_id, true));
                    black_key_id += 1;
                } else {
                    keys.push(Key::new(white_key_id, false));
                    white_key_id += 1;
                }
            }
        }

        keyboard_pipeline.update_instance_buffer(
            &mut target.gpu.encoder,
            &target.gpu.device,
            vec![QuadInstance::default(); 88],
        );

        let mut piano_keyboard = Self {
            keyboard_pipeline,
            keys,

            white_key_count: 52,
            black_key_count: 36,
        };

        piano_keyboard.resize(target);

        piano_keyboard
    }

    fn update_keys_instances(&mut self, target: &mut Target) {
        let keys = &self.keys;

        self.keyboard_pipeline.instances_mut(
            &mut target.gpu.encoder,
            &target.gpu.device,
            |instances| {
                for (_, key) in keys.iter().enumerate() {
                    instances[key.instance_id] = QuadInstance {
                        position: [key.pos.0, key.pos.1],
                        size: [key.size.0, key.size.1],
                        color: key.color.into_linear_rgba(),
                        border_radius: [0.0, 0.0, 7.0, 7.0],
                    };
                }
            },
        );
    }

    pub fn resize(&mut self, target: &mut Target) {
        let (window_w, window_h) = {
            let winit::dpi::LogicalSize { width, height } = target.window.state.logical_size;
            (width, height)
        };

        let white_width = window_w / self.white_key_count as f32;
        let white_height = window_h / 5.0;

        let mut white_key_id: usize = 0;

        for key in self.keys.iter_mut() {
            let x = white_key_id as f32 * white_width;
            let y = window_h - white_height;

            if key.is_black() {
                let black_width = white_width / 1.5;
                let black_height = white_height / 1.5;

                key.pos = (x - black_width / 2.0, y);
                key.size = (black_width, black_height);
            } else {
                key.pos = (x, y);
                key.size = (white_width - 1.0, white_height);
                white_key_id += 1;
            }
        }

        self.update_keys_instances(target);
    }

    pub fn update_notes_state(&mut self, target: &mut Target, notes: [(bool, usize); 88]) {
        let color_schema = &target.state.config.color_schema;

        for (id, (is_on, track)) in notes.iter().enumerate() {
            if *is_on {
                let color = &color_schema[track % color_schema.len()];
                self.keys[id].set_color(color);
            } else {
                self.keys[id].reset_color()
            }
        }

        self.update_keys_instances(target);
    }

    pub fn render<'rpass>(
        &'rpass mut self,
        transform_uniform: &'rpass Uniform<TransformUniform>,
        render_pass: &mut wgpu::RenderPass<'rpass>,
    ) {
        self.keyboard_pipeline
            .render(transform_uniform, render_pass);
    }
}
