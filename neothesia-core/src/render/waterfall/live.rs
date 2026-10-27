use std::{collections::HashMap, time::Duration};

use crate::{TransformUniform, Uniform, config::Config};
use midi_file::MidiNote;
use wgpu_jumpstart::Gpu;

use super::WaterfallRenderer;

/// Reversed waterfall built out of live played notes, notes grow out of the keyboard and rise up
pub struct LiveWaterfallRenderer {
    renderer: WaterfallRenderer,
    layout: piano_layout::KeyboardLayout,
    finished: Vec<MidiNote>,
    active: HashMap<u8, MidiNote>,
    time: Duration,
    queue: wgpu::Queue,
}

impl LiveWaterfallRenderer {
    pub fn new(
        gpu: &Gpu,
        config: &Config,
        transform_uniform: &Uniform<TransformUniform>,
        layout: piano_layout::KeyboardLayout,
    ) -> Self {
        let renderer =
            WaterfallRenderer::new(gpu, &[], &[], config, transform_uniform, layout.clone());

        let mut live = Self {
            renderer,
            layout: layout.clone(),
            finished: Vec::new(),
            active: HashMap::new(),
            time: Duration::ZERO,
            queue: gpu.queue.clone(),
        };
        live.resize(config, layout);
        live
    }

    pub fn resize(&mut self, config: &Config, layout: piano_layout::KeyboardLayout) {
        self.layout = layout;

        self.renderer
            .pipeline()
            .set_speed(&self.queue, -config.animation_speed().abs());
    }

    pub fn note_on(&mut self, channel: u8, key: u8, velocity: u8) {
        self.note_off(key);
        self.active.insert(
            key,
            MidiNote {
                start: self.time,
                end: self.time,
                duration: Duration::ZERO,
                note: key,
                velocity,
                channel,
                track_id: 0,
                track_color_id: 0,
            },
        );
    }

    pub fn note_off(&mut self, key: u8) {
        if let Some(note) = self.active.remove(&key) {
            self.finished.push(end_note(note, self.time));
        }
    }

    pub fn update(&mut self, config: &Config, delta: Duration, height: f32) {
        if self.finished.is_empty() && self.active.is_empty() {
            self.time = Duration::ZERO;
        } else {
            self.time += delta;
        }

        let time = self.time;
        let speed = self.renderer.pipeline().speed().abs();

        self.finished
            .retain(|note| (time - note.end).as_secs_f32() * speed < height);

        let mut notes: Vec<MidiNote> = self
            .finished
            .iter()
            .cloned()
            .chain(
                self.active
                    .values()
                    .map(|note| end_note(note.clone(), time)),
            )
            .filter(|note| self.layout.range.contains(note.note))
            .collect();
        let range_start = self.layout.range.start();
        notes.sort_unstable_by_key(|note| {
            let key = &self.layout.keys[(note.note - range_start) as usize];
            (key.kind().is_sharp(), note.start)
        });

        self.renderer.set_notes(notes, config, self.layout.clone());
        self.renderer.update(time.as_secs_f32());
    }

    #[profiling::function]
    pub fn render<'rpass>(&'rpass mut self, render_pass: &mut wgpu::RenderPass<'rpass>) {
        self.renderer.render(render_pass);
    }
}

fn end_note(mut note: MidiNote, time: Duration) -> MidiNote {
    note.end = time;
    note.duration = time - note.start;
    note
}
