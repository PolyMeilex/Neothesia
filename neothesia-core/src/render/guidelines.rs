use std::{sync::Arc, time::Duration};

use crate::{
    render::{QuadInstance, QuadRenderer},
    utils::Point,
};

pub struct GuidelineRenderer {
    pos: Point<f32>,

    layout: piano_layout::KeyboardLayout,
    vertical_guidelines: bool,
    horizontal_guidelines: bool,

    cache: Vec<QuadInstance>,
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
            measures,
        }
    }

    pub fn set_pos(&mut self, pos: Point<f32>) {
        self.pos = pos;
        self.cache.clear();
    }

    pub fn set_layout(&mut self, layout: piano_layout::KeyboardLayout) {
        self.layout = layout;
        self.cache.clear();
    }

    /// Reupload instances to GPU
    fn reupload(&mut self, size: dpi::LogicalSize<f32>) {
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
            let h = size.height;

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
        }
    }

    #[profiling::function]
    fn update_horizontal_guidelines(
        &mut self,
        quads: &mut QuadRenderer,
        animation_speed: f32,
        time: f32,
        size: dpi::LogicalSize<f32>,
    ) {
        for masure in self
            .measures
            .iter()
            .skip_while(|bar| bar.as_secs_f32() < time)
        {
            let x = 0.0;
            let y = self.pos.y - (masure.as_secs_f32() - time) * animation_speed;

            let w = size.width;
            let h = 1.0;

            if y < 0.0 {
                break;
            }

            quads.layer().push(QuadInstance {
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
        quads: &mut QuadRenderer,
        animation_speed: f32,
        scale: f32,
        time: f32,
        size: dpi::LogicalSize<f32>,
    ) {
        if self.cache.is_empty() {
            self.reupload(size);
        }

        if self.horizontal_guidelines {
            let animation_speed = animation_speed / scale;
            self.update_horizontal_guidelines(quads, animation_speed, time, size);
        }

        quads.layer().extend(&self.cache);
    }
}
