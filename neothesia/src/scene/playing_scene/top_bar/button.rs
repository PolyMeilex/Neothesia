use neothesia_core::{
    render::{QuadInstance, QuadPipeline, TextRenderer},
    Color,
};

use crate::scene::playing_scene::LAYER_FG;

pub struct Button {
    id: nuon::ElementId,
    is_hovered: bool,
    icon: &'static str,
    color: Color,
    hover_color: Color,
    rect: nuon::Rect,
}

impl Button {
    pub fn new(id: nuon::ElementId) -> Self {
        Self {
            id,
            is_hovered: false,
            icon: "",
            color: super::BAR_BG,
            hover_color: super::BUTTON_HOVER,
            rect: nuon::Rect::zero(),
        }
    }

    pub fn id(&self) -> nuon::ElementId {
        self.id
    }

    pub fn update<M>(
        &mut self,
        elements: &mut nuon::ElementsMap<M>,
        rect: nuon::Rect,
    ) -> &mut Self {
        if let Some(element) = elements.update(self.id(), rect) {
            self.update_state(element);
        }
        self
    }

    pub fn update_state<M>(&mut self, element: &nuon::Element<M>) -> &mut Self {
        self.set_hovered(element.hovered()).set_rect(element.rect())
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

    fn set_hovered(&mut self, hovered: bool) -> &mut Self {
        self.is_hovered = hovered;
        self
    }

    fn set_rect(&mut self, rect: nuon::Rect) -> &mut Self {
        self.rect = rect;
        self
    }

    pub fn set_icon(&mut self, icon: &'static str) -> &mut Self {
        self.icon = icon;
        self
    }

    pub fn draw(&self, quad_pipeline: &mut QuadPipeline, text: &mut TextRenderer) {
        let color = if self.is_hovered {
            self.hover_color
        } else {
            self.color
        }
        .into_linear_rgba();

        quad_pipeline.push(
            LAYER_FG,
            QuadInstance {
                position: self.rect.origin.into(),
                size: self.rect.size.into(),
                color,
                border_radius: [5.0; 4],
            },
        );

        let icon_size = 20.0;
        text.queue_icon(
            self.rect.origin.x + (self.rect.size.width - icon_size) / 2.0,
            self.rect.origin.y + (self.rect.size.height - icon_size) / 2.0,
            icon_size,
            self.icon,
        );
    }
}
