use super::keyboard_pipeline::{KeyInstance, KeyStateInstance, KeyboardPipeline};
use crate::wgpu_jumpstart::Gpu;
use crate::MainState;

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
    pub x: f32,
    pub w: f32,
    pub is_black: bool,
}

pub struct PianoKeyboard {
    pub keyboard_pipeline: KeyboardPipeline,
    pub all_keys: Vec<Key>,
}

impl PianoKeyboard {
    pub fn new(state: &MainState, gpu: &mut Gpu) -> Self {
        let keyboard_pipeline = KeyboardPipeline::new(state, gpu);
        let mut piano_keyboard = Self {
            keyboard_pipeline,
            all_keys: Vec::new(),
        };
        piano_keyboard.resize(state, gpu);

        piano_keyboard
    }
    pub fn resize(&mut self, state: &crate::MainState, gpu: &mut Gpu) {
        let w = state.window_size.0 / 52.0;
        let h = 120.0 * state.window_size.0 / state.window_size.1;

        let mut x_offset = 0.0;

        self.all_keys.clear();
        let mut white_keys = Vec::new();
        let mut black_keys = Vec::new();

        let mut rectangles = Vec::new();
        for id in 0..88 {
            let x = id as f32 * w;
            let y = 0.0;

            let key_id = id + 9;
            let note_id = key_id % 12;

            if note_id == KEY_CIS
                || note_id == KEY_DIS
                || note_id == KEY_FIS
                || note_id == KEY_GIS
                || note_id == KEY_AIS
            {
                x_offset -= w;

                let w = w / 1.5;
                let h = h / 1.5;

                let black_offset = w;

                let x = x_offset + black_offset + x + w / 2.0;
                let y = y + h / 2.0 + state.window_size.1 - h * 1.5;

                self.all_keys.push(Key {
                    x,
                    w,
                    is_black: true,
                });
                black_keys.push((x, y, w, h));
            } else {
                let x = x_offset + x + w / 2.0;
                let y = y + h / 2.0 + state.window_size.1 - h;

                self.all_keys.push(Key {
                    x,
                    w,
                    is_black: false,
                });
                white_keys.push((x, y, w, h));
            }
        }

        // To lazy to use depth buffer so we draw white keys first
        for rect in white_keys {
            rectangles.push(KeyInstance {
                position: [rect.0, rect.1],
                size: [rect.2 - 1.0, rect.3],
                is_black: 0,
                radius: 5.0 * state.window_size.0 / state.window_size.1,
            });
        }
        for rect in black_keys {
            rectangles.push(KeyInstance {
                position: [rect.0, rect.1],
                size: [rect.2 - 1.0, rect.3],
                is_black: 1,
                radius: 5.0 * state.window_size.0 / state.window_size.1,
            });
        }

        self.keyboard_pipeline
            .update_instance_buffer(gpu, rectangles);
    }
    pub fn update_notes(&mut self, gpu: &mut Gpu, notes: [bool; 88]) {
        let mut white_keys = Vec::new();
        let mut black_keys = Vec::new();

        // Becouse white keys are first in instance bufer we need to split input
        for id in 0..88 {
            let key_id = id + 9;
            let note_id = key_id % 12;
            let on = notes[id as usize];

            if note_id == KEY_CIS
                || note_id == KEY_DIS
                || note_id == KEY_FIS
                || note_id == KEY_GIS
                || note_id == KEY_AIS
            {
                black_keys.push(on);
            } else {
                white_keys.push(on);
            }
        }

        let notes_out = white_keys
            .into_iter()
            .chain(black_keys.into_iter())
            .map(|b| if b { 1 } else { 0 })
            .map(|b| KeyStateInstance { on: b })
            .collect();

        self.keyboard_pipeline
            .update_notes_state(&mut gpu.encoder, &gpu.device, notes_out);
    }
    pub fn render(&mut self, state: &MainState, gpu: &mut Gpu, frame: &wgpu::SwapChainOutput) {
        let encoder = &mut gpu.encoder;
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &frame.view,
                    resolve_target: None,
                    load_op: wgpu::LoadOp::Load,
                    store_op: wgpu::StoreOp::Store,
                    clear_color: wgpu::Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 0.0,
                    },
                }],
                depth_stencil_attachment: None,
            });
            self.keyboard_pipeline.render(state, &mut render_pass);
        }
    }
}
