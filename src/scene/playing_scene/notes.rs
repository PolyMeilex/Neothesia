use super::notes_pipeline::{NoteInstance, NotesPipeline};
use crate::wgpu_jumpstart::{Color, Gpu};
use crate::Target;
use crate::TransformUniform;
use crate::Uniform;

pub struct Notes {
    notes_pipeline: NotesPipeline,
}

impl Notes {
    pub fn new(target: &mut Target, keys: &[super::keyboard::Key]) -> Self {
        let notes_pipeline = NotesPipeline::new(target, target.state.midi_file.as_ref().unwrap());
        let mut notes = Self { notes_pipeline };
        notes.resize(target, keys);
        notes
    }
    pub fn resize(&mut self, target: &mut Target, keys: &[super::keyboard::Key]) {
        let midi = &target.state.midi_file.as_ref().unwrap();

        let (window_w, window_h) = {
            let winit::dpi::LogicalSize { width, height } = target.window.state.logical_size;
            (width, height)
        };

        let mut instances = Vec::new();

        let mut longer_than_88 = false;
        for note in midi.merged_track.notes.iter() {
            if note.note >= 21 && note.note <= 108 && note.ch != 9 {
                let key = &keys[note.note as usize - 21];
                let ar = window_w / window_h;

                let color_schema = &target.state.config.color_schema;

                let color = &color_schema[note.track_id % color_schema.len()];
                let color = if key.is_black { color.dark } else { color.base };
                let color: Color = color.into();

                let h = if note.duration >= 0.1 {
                    note.duration
                } else {
                    0.1
                };

                instances.push(NoteInstance {
                    position: [key.x, note.start],
                    size: [key.w - 1.0, h - 0.01], // h - 0.01 to make a litle gap bettwen successive notes
                    color: color.into_linear_rgb(),
                    radius: 4.0 * ar,
                });
            } else {
                longer_than_88 = true;
            }
        }

        if longer_than_88 {
            log::warn!("Midi Wider Than 88 Keys!");
        }

        self.notes_pipeline
            .update_instance_buffer(&mut target.gpu, instances);
    }
    pub fn update(&mut self, target: &mut Target, time: f32) {
        self.notes_pipeline.update_time(&mut target.gpu, time);
    }
    pub fn render<'rpass>(
        &'rpass mut self,
        transform_uniform: &'rpass Uniform<TransformUniform>,
        render_pass: &mut wgpu::RenderPass<'rpass>,
    ) {
        self.notes_pipeline.render(transform_uniform, render_pass);
    }
}
