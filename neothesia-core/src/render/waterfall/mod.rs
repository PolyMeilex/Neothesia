use std::{collections::HashMap, rc::Rc};

use crate::{
    TransformUniform, Uniform,
    config::{ColorSchemaV1, Config},
};
use midi_file::{MidiNote, MidiTrack};
use piano_layout::{Key, KeyKind, KeyboardLayout};
use wgpu_jumpstart::{Color, Gpu};

mod pipeline;
use pipeline::{NoteInstance, WaterfallPipeline};

#[derive(Clone)]
pub struct NoteList {
    pub(crate) inner: Rc<[MidiNote]>,
}

impl NoteList {
    fn new(tracks: &[MidiTrack], hidden_tracks: &[usize]) -> Self {
        let mut notes: Vec<_> = tracks
            .iter()
            .filter(|track| !hidden_tracks.contains(&track.track_id))
            .flat_map(|track| track.notes.iter().cloned())
            .collect();

        notes.sort_unstable_by(|a, b| {
            // We want to render newer or sharp notes on top of other notes
            match a.is_sharp().cmp(&b.is_sharp()) {
                std::cmp::Ordering::Equal => a.start.cmp(&b.start),
                other => other,
            }
        });

        Self {
            inner: notes.into(),
        }
    }

    fn len(&self) -> usize {
        self.inner.len()
    }
}

pub struct WaterfallRenderer {
    notes_pipeline: WaterfallPipeline,

    /// Notes that are in the process of being built (during free-play)
    staging_notes_in_proggress: HashMap<u8, usize>,
    staging_notes_pipeline: WaterfallPipeline,
    last_time: f32,

    notes: NoteList,
    device: wgpu::Device,
    queue: wgpu::Queue,
    color_schema: Vec<ColorSchemaV1>,
    layout: piano_layout::KeyboardLayout,
}

fn note_width(width: f32) -> f32 {
    // -1.0 to make a little gap bettwen neighbour notes
    width - 1.0
}

fn note_height(height: f32) -> f32 {
    let h = height.max(0.1);
    // -0.01 to make a little gap bettwen successive notes
    h - 0.01
}

fn note_radius(width: f32) -> f32 {
    width * 0.2
}

fn note_color(kind: KeyKind, color: &ColorSchemaV1) -> (u8, u8, u8) {
    if kind.is_sharp() {
        color.dark
    } else {
        color.base
    }
}

fn layout_key(layout: &KeyboardLayout, note: u8) -> Option<&Key> {
    let range_start = layout.range.start() as usize;
    if layout.range.contains(note) {
        Some(&layout.keys[note as usize - range_start])
    } else {
        None
    }
}

impl WaterfallRenderer {
    pub fn new(
        gpu: &Gpu,
        tracks: &[MidiTrack],
        hidden_tracks: &[usize],
        config: &Config,
        transform_uniform: &Uniform<TransformUniform>,
        layout: piano_layout::KeyboardLayout,
    ) -> Self {
        let notes = NoteList::new(tracks, hidden_tracks);

        let staging_notes_pipeline = WaterfallPipeline::new(gpu, transform_uniform, 32);

        let notes_pipeline = WaterfallPipeline::new(gpu, transform_uniform, notes.len());
        let mut notes = Self {
            notes_pipeline,
            staging_notes_pipeline,
            staging_notes_in_proggress: HashMap::new(),
            last_time: 0.0,
            notes,
            device: gpu.device.clone(),
            queue: gpu.queue.clone(),
            color_schema: config.color_schema().to_vec(),
            layout: layout.clone(),
        };
        notes
            .notes_pipeline
            .set_speed(&gpu.queue, config.animation_speed());
        notes
            .staging_notes_pipeline
            .set_speed(&gpu.queue, config.animation_speed());
        notes.resize(config, layout);
        notes
    }

    pub fn pipeline(&mut self) -> &mut WaterfallPipeline {
        &mut self.notes_pipeline
    }

    pub fn notes(&self) -> &NoteList {
        &self.notes
    }

    pub fn resize(&mut self, config: &Config, layout: piano_layout::KeyboardLayout) {
        let range_start = layout.range.start() as usize;

        self.notes_pipeline.clear();

        let mut longer_than_range = false;
        for note in self.notes.inner.iter() {
            if let Some(key) = layout_key(&layout, note.note) {
                if note.channel != 9 {
                    let color_schema = config.color_schema();

                    let color = &color_schema[note.track_color_id % color_schema.len()];
                    let color: Color = note_color(*key.kind(), color).into();
                    let width = key.width();

                    self.notes_pipeline.instances().push(NoteInstance {
                        position: [key.x(), note.start.as_secs_f32()],
                        size: [note_width(width), note_height(note.duration.as_secs_f32())],
                        color: color.into_linear_rgb(),
                        radius: note_radius(width),
                    });
                }
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

        self.notes_pipeline.set_diry();
        self.layout = layout;
    }

    pub fn push_note(&mut self, start: f32, note: u8) {
        // Staring a note should auto stop the previous one
        if self.staging_notes_in_proggress.contains_key(&note) {
            self.pop_note(note);
        }

        let Some(key) = layout_key(&self.layout, note) else {
            return;
        };

        let color = &self.color_schema[if note == 0 { 0 } else { 1 }];
        let color: Color = note_color(*key.kind(), color).into();

        let width = key.width();
        let x = key.x();

        let idx = self.staging_notes_pipeline.instances().len();
        self.staging_notes_pipeline.instances().push(NoteInstance {
            position: [x, start],
            size: [note_width(width), 0.0],
            color: color.into_linear_rgb(),
            radius: note_radius(width),
        });
        self.staging_notes_pipeline.set_diry();
        self.staging_notes_in_proggress.insert(note, idx);
    }

    pub fn pop_note(&mut self, note: u8) {
        let Some(idx) = self.staging_notes_in_proggress.remove(&note) else {
            return;
        };

        let mut note = self.staging_notes_pipeline.instances().remove(idx);
        self.staging_notes_pipeline.set_diry();

        if note.size[1] <= 0.0 {
            return;
        }

        for (_note, note_idx) in self.staging_notes_in_proggress.iter_mut() {
            if *note_idx >= idx {
                *note_idx -= 1;
            }
        }

        note.size[1] = note_height(note.size[1]);

        self.notes_pipeline.instances().push(note);
        self.notes_pipeline.set_diry();
    }

    fn update_staging_notes(&mut self, time: f32) {
        if self.staging_notes_pipeline.is_empty() {
            return;
        }

        self.staging_notes_pipeline.set_diry();
        for note in self.staging_notes_pipeline.instances() {
            note.size[1] += time - self.last_time;
        }
    }

    pub fn update(&mut self, time: f32) {
        self.notes_pipeline.update_time(&self.queue, time);
        self.staging_notes_pipeline.update_time(&self.queue, time);
        self.update_staging_notes(time);
        self.last_time = time;

        if self.notes_pipeline.is_diry() {
            self.notes_pipeline.prepare(&self.device, &self.queue);
        }
        if self.staging_notes_pipeline.is_diry() {
            self.staging_notes_pipeline
                .prepare(&self.device, &self.queue);
        }
    }

    #[profiling::function]
    pub fn render<'rpass>(&'rpass mut self, render_pass: &mut wgpu::RenderPass<'rpass>) {
        self.notes_pipeline.render(render_pass);
        if !self.staging_notes_pipeline.is_empty() {
            self.staging_notes_pipeline.render(render_pass);
        }
    }
}
