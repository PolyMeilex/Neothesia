use neothesia_core::{
    render::{QuadInstance, QuadPipeline, TextRenderer},
    utils::{Bbox, Point, Size},
    Color,
};

use crate::scene::playing_scene::LAYER_FG;

pub struct Button {
    bbox: Bbox,
    is_hovered: bool,
    icon: &'static str,
    color: Color,
    hover_color: Color,
}

impl Default for Button {
    fn default() -> Self {
        Self::new()
    }
}

impl Button {
    pub fn new() -> Self {
        Self {
            bbox: Bbox::new(Point::new(0.0, 0.0), Size::new(30.0, 30.0)),
            is_hovered: false,
            icon: "",
            color: super::BAR_BG,
            hover_color: super::BUTTON_HOVER,
        }
    }

    #[allow(dead_code)]
    pub fn set_color(&mut self, color: Color) -> &mut Self {
        self.color = color;
        self
    }

    #[allow(dead_code)]
    pub fn set_hover_color(&mut self, color: Color) -> &mut Self {
        self.hover_color = color;
        self
    }

    pub fn set_pos(&mut self, pos: impl Into<Point<f32>>) -> &mut Self {
        self.bbox.pos = pos.into();
        self
    }

    #[allow(dead_code)]
    pub fn set_x(&mut self, x: f32) -> &mut Self {
        self.bbox.pos.x = x;
        self
    }

    #[allow(dead_code)]
    pub fn set_y(&mut self, y: f32) -> &mut Self {
        self.bbox.pos.y = y;
        self
    }

    pub fn set_hovered(&mut self, hovered: bool) -> &mut Self {
        self.is_hovered = hovered;
        self
    }

    pub fn set_icon(&mut self, icon: &'static str) -> &mut Self {
        self.icon = icon;
        self
    }

    pub fn bbox(&self) -> &Bbox {
        &self.bbox
    }

    pub fn draw(&mut self, quad_pipeline: &mut QuadPipeline, text: &mut TextRenderer) {
        let color = if self.is_hovered {
            self.hover_color
        } else {
            self.color
        }
        .into_linear_rgba();

        quad_pipeline.push(
            LAYER_FG,
            QuadInstance {
                position: self.bbox.pos.into(),
                size: self.bbox.size.into(),
                color,
                border_radius: [5.0; 4],
            },
        );

        let icon_size = 20.0;
        text.queue_icon(
            self.bbox.x() + (self.bbox.w() - icon_size) / 2.0,
            self.bbox.y() + (self.bbox.h() - icon_size) / 2.0,
            icon_size,
            self.icon,
        );
    }
}
