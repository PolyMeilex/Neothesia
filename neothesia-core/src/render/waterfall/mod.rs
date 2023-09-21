use crate::config::Config;
use crate::TransformUniform;
use crate::Uniform;
use midi_file::Midi;
use wgpu_jumpstart::Color;
use wgpu_jumpstart::Gpu;

mod pipeline;
use pipeline::{NoteInstance, WaterfallPipeline};

pub struct WaterfallRenderer {
    notes_pipeline: WaterfallPipeline,
}

impl WaterfallRenderer {
    pub fn new(
        gpu: &Gpu,
        midi: &Midi,
        config: &Config,
        transform_uniform: &Uniform<TransformUniform>,
        layout: piano_math::KeyboardLayout,
    ) -> Self {
        let notes_pipeline =
            WaterfallPipeline::new(gpu, transform_uniform, midi.merged_track.notes.len());
        let mut notes = Self { notes_pipeline };
        notes
            .notes_pipeline
            .set_speed(&gpu.queue, config.animation_speed);
        notes.resize(&gpu.queue, midi, config, layout);
        notes
    }

    pub fn pipeline(&mut self) -> &mut WaterfallPipeline {
        &mut self.notes_pipeline
    }

    pub fn resize(
        &mut self,
        queue: &wgpu::Queue,
        midi: &Midi,
        config: &Config,
        layout: piano_math::KeyboardLayout,
    ) {
        let range_start = layout.range.start() as usize;

        let mut instances = Vec::new();

        let mut longer_than_range = false;
        for note in midi.merged_track.notes.iter() {
            if layout.range.contains(note.note) && note.channel != 9 {
                let key = &layout.keys[note.note as usize - range_start];

                let color_schema = &config.color_schema;

                let color = &color_schema[note.track_color_id % color_schema.len()];
                let color = if key.kind().is_sharp() {
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
                    position: [key.x(), note.start.as_secs_f32()],
                    size: [key.width() - 1.0, h - 0.01], // h - 0.01 to make a litle gap bettwen successive notes
                    color: color.into_linear_rgb(),
                    radius: key.width() * 0.2,
                });
            } else {
                longer_than_range = true;
            }
        }

        if longer_than_range {
            log::warn!(
                "Midi wider than giver range: {range_start}-{}",
                layout.range.end()
            );
        }

        self.notes_pipeline.update_instance_buffer(queue, instances);
    }

    pub fn update(&mut self, queue: &wgpu::Queue, time: f32) {
        self.notes_pipeline.update_time(queue, time);
    }

    pub fn render<'rpass>(
        &'rpass mut self,
        transform_uniform: &'rpass Uniform<TransformUniform>,
        render_pass: &mut wgpu::RenderPass<'rpass>,
    ) {
        self.notes_pipeline.render(transform_uniform, render_pass);
    }
}
