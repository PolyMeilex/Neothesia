pub struct NeoBtn {
    id: Option<nuon::Id>,
    size: nuon::Size,
    color: nuon::Color,
    label: String,
    icon: String,
    tooltip: String,
}

impl NeoBtn {
    pub fn new() -> Self {
        Self {
            id: None,
            size: Default::default(),
            color: nuon::Color::WHITE,
            label: Default::default(),
            icon: Default::default(),
            tooltip: Default::default(),
        }
    }

    #[allow(unused)]
    pub fn id(mut self, id: impl Into<nuon::Id>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.size = (width, height).into();
        self
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = icon.into();
        self
    }

    pub fn tooltip(mut self, tooltip: impl Into<String>) -> Self {
        self.tooltip = tooltip.into();
        self
    }

    pub fn color(mut self, color: impl Into<nuon::Color>) -> Self {
        self.color = color.into();
        self
    }

    pub fn build(&self, ui: &mut nuon::Ui) -> bool {
        let w = self.size.width;
        let h = self.size.height;

        let id = if let Some(id) = self.id {
            id
        } else if self.icon.is_empty() {
            nuon::Id::hash(&self.label)
        } else {
            nuon::Id::hash(&self.icon)
        };

        let event = nuon::click_area(id).size(w, h).build(ui);

        let (bg, accent) = if event.is_hovered() || event.is_pressed() {
            (
                nuon::Color::new_u8(9, 9, 9, 0.6),
                nuon::Color::new_u8(56, 145, 255, 1.0),
            )
        } else {
            (
                nuon::Color::new_u8(17, 17, 17, 0.6),
                nuon::Color::new_u8(160, 81, 255, 1.0),
            )
        };

        nuon::quad()
            .size(w, h)
            .color(bg)
            .border_radius([7.0; 4])
            .build(ui);
        nuon::quad()
            .size(w, 7.0)
            .y(h - 7.0)
            .color(accent)
            .border_radius([0.0, 0.0, 7.0, 7.0])
            .build(ui);

        let label = nuon::label()
            .size(self.size.width, self.size.height)
            .font_size(30.0)
            .color(self.color);
        if self.icon.is_empty() {
            label.text(&self.label).build(ui);
        } else {
            label.icon(&self.icon).build(ui);
        }

        if event.is_hovered() || event.is_pressed() {
            nuon::label()
                .text(&self.tooltip)
                .size(w, 13.0)
                .y(-13.0)
                .build(ui);
        }

        event.is_clicked()
    }
}

pub fn neo_btn() -> NeoBtn {
    NeoBtn::new()
}

pub fn neo_btn_icon(ui: &mut nuon::Ui, w: f32, h: f32, icon: &str) -> bool {
    NeoBtn::new().size(w, h).icon(icon).build(ui)
}
