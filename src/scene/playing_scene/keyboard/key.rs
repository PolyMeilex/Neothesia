use crate::{
    config::ColorSchema,
    utils::{Point, Size},
};
use neothesia_pipelines::quad::QuadInstance;
use wgpu_jumpstart::Color;

pub struct Key {
    pub(super) pos: Point<f32>,
    pub(super) size: Size<f32>,
    pub(super) is_black: bool,
    pub(super) note_id: u8,

    pub(super) color: Color,
}

impl Key {
    pub fn new(is_black: bool) -> Self {
        Self {
            pos: Default::default(),
            size: Default::default(),
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
        self.pos.x
    }

    pub fn width(&self) -> f32 {
        self.size.w
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
        let kind_multiplier = if key.is_black() { 1.0 } else { 3.5 };

        let radius = key.size.w * 0.08;
        let radius = radius * kind_multiplier;

        QuadInstance {
            position: key.pos.into(),
            size: key.size.into(),
            color: key.color.into_linear_rgba(),
            border_radius: [0.0, 0.0, radius, radius],
        }
    }
}
