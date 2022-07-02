use crate::target::Target;
use crate::TransformUniform;
use crate::Uniform;
use waterfall_pipeline::{NoteInstance, WaterfallPipeline};
use wgpu_jumpstart::Color;

pub struct Notes {
    notes_pipeline: WaterfallPipeline,
}

impl Notes {
    pub fn new(target: &mut Target, keys: &[super::keyboard::Key]) -> Self {
        let notes_pipeline = WaterfallPipeline::new(
            &target.gpu,
            &target.transform_uniform,
            target.midi_file.as_ref().unwrap().merged_track.notes.len(),
        );
        let mut notes = Self { notes_pipeline };
        notes.resize(target, keys);
        notes
    }

    pub fn resize(&mut self, target: &mut Target, keys: &[super::keyboard::Key]) {
        let midi = &target.midi_file.as_ref().unwrap();

        let (window_w, window_h) = {
            let winit::dpi::LogicalSize { width, height } = target.window.state.logical_size;
            (width, height)
        };

        let mut instances = Vec::new();

        let mut longer_than_88 = false;
        for note in midi.merged_track.notes.iter() {
            if note.note >= 21 && note.note <= 108 && note.channel != 9 {
                let key = &keys[note.note as usize - 21];
                let ar = window_w / window_h;

                let color_schema = &target.config.color_schema;

                let color = &color_schema[note.track_id % color_schema.len()];
                let color = if key.is_black() {
                    color.dark
                } else {
                    color.base
                };
                let color: Color = color.into();

                let h = if note.duration.as_secs_f32() >= 0.1 {
                    note.duration.as_secs_f32()
                } else {
                    0.1
                };

                instances.push(NoteInstance {
                    position: [key.x_position(), note.start.as_secs_f32()],
                    size: [key.width() - 1.0, h - 0.01], // h - 0.01 to make a litle gap bettwen successive notes
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
