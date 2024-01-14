use crate::{
    render::{QuadInstance, QuadPipeline},
    utils::Point,
};

pub struct GuidelineRenderer {
    pos: Point<f32>,

    layout: piano_math::KeyboardLayout,
    vertical_guidelines: bool,

    cache: Vec<QuadInstance>,
}

impl GuidelineRenderer {
    pub fn new(
        layout: piano_math::KeyboardLayout,
        pos: Point<f32>,
        vertical_guidelines: bool,
    ) -> Self {
        Self {
            pos,
            layout,
            vertical_guidelines,
            cache: Vec::new(),
        }
    }

    pub fn set_pos(&mut self, pos: Point<f32>) {
        self.pos = pos;
        self.cache.clear();
    }

    pub fn set_layout(&mut self, layout: piano_math::KeyboardLayout) {
        self.layout = layout;
        self.cache.clear();
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
        }
    }

    pub fn update(&mut self, quads: &mut QuadPipeline) {
        if self.cache.is_empty() {
            self.reupload();
        }

        for quad in self.cache.iter() {
            quads.instances().push(*quad);
        }
    }
}
