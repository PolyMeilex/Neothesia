use crate::config::ColorSchema;
use neothesia_pipelines::quad::QuadInstance;
use wgpu_jumpstart::Color;

pub struct Key {
    pub(super) pos: (f32, f32),
    pub(super) size: (f32, f32),
    pub(super) is_black: bool,
    pub(super) note_id: u8,

    pub(super) color: Color,
}

impl Key {
    pub fn new(is_black: bool) -> Self {
        Self {
            pos: (0.0, 0.0),
            size: (0.0, 0.0),
            is_black,
            note_id: 0,

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

impl From<&Key> for QuadInstance {
    fn from(key: &Key) -> QuadInstance {
        QuadInstance {
            position: [key.pos.0, key.pos.1],
            size: [key.size.0, key.size.1],
            color: key.color.into_linear_rgba(),
            border_radius: if key.is_black() {
                [0.0, 0.0, 2.0, 2.0]
            } else {
                [0.0, 0.0, 7.0, 7.0]
            },
        }
    }
}
