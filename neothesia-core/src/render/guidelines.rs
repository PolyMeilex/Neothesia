use std::{sync::Arc, time::Duration};

use crate::{
    render::{QuadInstance, QuadPipeline},
    render::text::{TextInstance, TextPipeline}, // Move to text module
    utils::Point,
};

pub struct GuidelineRenderer {
    pos: Point<f32>,

    layout: piano_layout::KeyboardLayout,
    vertical_guidelines: bool,
    horizontal_guidelines: bool,

    cache: Vec<QuadInstance>,
    text_cache: Vec<TextInstance>,
    measures: Arc<[Duration]>,
}

impl GuidelineRenderer {
    pub fn new(
        layout: piano_layout::KeyboardLayout,
        pos: Point<f32>,
        vertical_guidelines: bool,
        horizontal_guidelines: bool,
        measures: Arc<[Duration]>,
    ) -> Self {
        Self {
            pos,
            layout,
            vertical_guidelines,
            horizontal_guidelines,
            cache: Vec::new(),
            text_cache: Vec::new(),
            measures,
        }
    }

    pub fn set_pos(&mut self, pos: Point<f32>) {
        self.pos = pos;
        self.cache.clear();
        self.text_cache.clear();
    }

    pub fn set_layout(&mut self, layout: piano_layout::KeyboardLayout) {
        self.layout = layout;
        self.cache.clear();
        self.text_cache.clear();
    }

    /// Reupload instances to GPU
    fn reupload(&mut self) {
        if !self.vertical_guidelines {
            return;
        }

        for key in self
            .layout
            .keys
            .iter()
            .filter(|key| key.note_id() == 0 || key.note_id() == 5)
        {
            let x = self.pos.x + key.x();
            let y = 0.0;

            let w = 1.0;
            let h = f32::MAX;

            let color = if key.note_id() == 0 {
                [0.2, 0.2, 0.2, 1.0]
            } else {
                [0.05, 0.05, 0.05, 1.0]
            };

            self.cache.push(QuadInstance {
                position: [x, y],
                size: [w, h],
                color,
                border_radius: [0.0, 0.0, 0.0, 0.0],
            });

            // Add text instance for note label
            self.text_cache.push(TextInstance {
                position: [x, y],
                text: key.note_id().to_string(),
                color: [1.0, 1.0, 1.0, 1.0],
                scale: 1.0,
            });
        }
    }

    // Helper method to convert key index to note name
    fn get_note_name(key_index: usize) -> &'static str {
        const NOTE_NAMES: [&str; 12] = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
        NOTE_NAMES[key_index % 12]
    }

    #[profiling::function]
    fn update_vertical_guidelines(&mut self, quads: &mut QuadPipeline, layer: usize) {
        for key in self.layout.white_keys().chain(self.layout.black_keys()) {
            let x = self.pos.x + key.x();
            let y = self.pos.y;

            // Add the quad instance
            quads.instances(layer).push(QuadInstance {
                position: [x, y],                    // Position of the guideline
                size: [2.0, self.layout.height()],   // Thin width, full keyboard height
                color: [0.2, 0.2, 0.2, 0.5],        // Semi-transparent dark gray
                border_radius: [0.0, 0.0, 0.0, 0.0], // Fix: Use [f32; 4] instead of f32
            });

            // Add text label
            self.text_cache.push(TextInstance {
                position: [x + 2.0, y + 2.0], // Slight offset from quad corner
                text: Self::get_note_name(key.index()).to_string(),
                color: [1.0, 1.0, 1.0, 1.0], // White text
                scale: 0.8,
            });
        }
    }

    #[profiling::function]
    fn update_horizontal_guidelines(
        &mut self,
        quads: &mut QuadPipeline,
        layer: usize,
        animation_speed: f32,
        time: f32,
    ) {
        for masure in self
            .measures
            .iter()
            .skip_while(|bar| bar.as_secs_f32() < time)
        {
            let x = 0.0;
            let y = self.pos.y - (masure.as_secs_f32() - time) * animation_speed;

            let w = f32::MAX;
            let h = 1.0;

            if y < 0.0 {
                break;
            }

            quads.instances(layer).push(QuadInstance {
                position: [x, y],
                size: [w, h],
                color: [0.05, 0.05, 0.05, 1.0],
                border_radius: [0.0, 0.0, 0.0, 0.0],
            });
        }
    }

    #[profiling::function]
    pub fn update(
        &mut self,
        quads: &mut QuadPipeline,
        texts: &mut TextPipeline,
        layer: usize,
        animation_speed: f32,
        time: f32,
    ) {
        if self.cache.is_empty() {
            self.reupload();
        }

        if self.horizontal_guidelines {
            self.update_horizontal_guidelines(quads, layer, animation_speed, time);
        }

        for quad in self.cache.iter() {
            quads.instances(layer).push(*quad);
        }

        for text in self.text_cache.iter() {
            texts.instances(layer).push(*text);
        }
    }
}

impl KeyboardLayout {
    pub fn white_keys(&self) -> impl Iterator<Item = &Key> {
        self.keys.iter().filter(|k| !k.is_black())
    }

    pub fn black_keys(&self) -> impl Iterator<Item = &Key> {
        self.keys.iter().filter(|k| k.is_black())
    }

    pub fn height(&self) -> f32 {
        self.height
    }
}

