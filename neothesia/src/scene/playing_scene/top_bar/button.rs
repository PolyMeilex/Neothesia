use neothesia_core::{
    render::{QuadInstance, QuadPipeline, TextRenderer},
    Color,
};

use crate::scene::playing_scene::LAYER_FG;

pub struct Button {
    id: nuon::ElementId,
    icon: &'static str,
    color: Color,
    hover_color: Color,
}

impl Button {
    pub fn new<M>(elements: &mut nuon::ElementsMap<M>, builder: nuon::ElementBuilder<M>) -> Self {
        Self {
            id: elements.insert(builder),
            icon: "",
            color: super::BAR_BG,
            hover_color: super::BUTTON_HOVER,
        }
    }

    pub fn id(&self) -> nuon::ElementId {
        self.id
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

    pub fn set_icon(&mut self, icon: &'static str) -> &mut Self {
        self.icon = icon;
        self
    }

    pub fn draw<M>(
        &self,
        quad_pipeline: &mut QuadPipeline,
        text: &mut TextRenderer,
        element: &nuon::Element<M>,
    ) {
        let is_hovered = element.hovered();
        let rect = element.rect();

        let color = if is_hovered {
            self.hover_color
        } else {
            self.color
        }
        .into_linear_rgba();

        quad_pipeline.push(
            LAYER_FG,
            QuadInstance {
                position: rect.origin.into(),
                size: rect.size.into(),
                color,
                border_radius: [5.0; 4],
            },
        );

        let icon_size = 20.0;
        text.queue_icon(
            rect.origin.x + (rect.size.width - icon_size) / 2.0,
            rect.origin.y + (rect.size.height - icon_size) / 2.0,
            icon_size,
            self.icon,
        );
    }
}
