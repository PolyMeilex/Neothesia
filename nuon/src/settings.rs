use crate::{self as nuon, TextJustify, Ui};

pub struct SettingsSection<'a> {
    label: String,
    width: f32,
    rows: Vec<SettingsRow<'a>>,
}

impl<'a> SettingsSection<'a> {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            width: 0.0,
            rows: Vec::new(),
        }
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    pub fn row(mut self, row: SettingsRow<'a>) -> Self {
        self.rows.push(row);
        self
    }

    pub fn build(self, ui: &mut Ui) {
        let spacer_label_h = 43.0;

        let spacer = |ui: &mut Ui| {
            nuon::quad()
                .width(self.width)
                .height(1.0)
                .color([0; 3])
                .build(ui);
            nuon::translate().y(1.0).add_to_current(ui);
        };

        nuon::label()
            .text(self.label)
            .size(self.width, spacer_label_h)
            .font_size(14.6)
            .text_justify(TextJustify::Left)
            .bold(true)
            .build(ui);

        nuon::translate().y(spacer_label_h).add_to_current(ui);

        let pos = nuon::row_group().build(ui, |ui| {
            let len = self.rows.len();
            for (id, row) in self.rows.into_iter().enumerate() {
                row.width(self.width).build(ui);
                if id + 1 < len {
                    spacer(ui);
                }
            }

            nuon::translate().x(self.width).add_to_current(ui);
        });

        nuon::translate().y(pos.y).add_to_current(ui);
    }
}

pub fn settings_section<'a>(label: impl Into<String>) -> SettingsSection<'a> {
    SettingsSection::new(label)
}

pub struct SettingsRow<'a> {
    title: String,
    subtitle: String,
    width: f32,
    #[allow(clippy::type_complexity)]
    body: Box<dyn FnOnce(&mut Ui, f32, f32) + 'a>,
}

impl<'a> Default for SettingsRow<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> SettingsRow<'a> {
    pub fn new() -> Self {
        Self {
            title: String::new(),
            subtitle: String::new(),
            width: 0.0,
            body: Box::new(|_, _, _| {}),
        }
    }

    fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    pub fn title(mut self, label: impl Into<String>) -> Self {
        self.title = label.into();
        self
    }

    pub fn subtitle(mut self, label: impl Into<String>) -> Self {
        self.subtitle = label.into();
        self
    }

    pub fn body(mut self, build: impl FnOnce(&mut Ui, f32, f32) + 'a) -> Self {
        self.body = Box::new(build);
        self
    }

    fn build(self, ui: &mut Ui) {
        let row_h = 54.0;
        let row_padding = 15.0;
        let row_inner_w = self.width - 2.0 * row_padding;

        nuon::translate().x(row_padding).build(ui, |ui| {
            let title_h = 14.6;
            let subtitle_h = 12.2;

            if self.subtitle.is_empty() {
                nuon::label()
                    .text(self.title)
                    .text_justify(nuon::TextJustify::Left)
                    .font_size(title_h)
                    .size(row_inner_w, row_h)
                    .build(ui);
            } else {
                let gap = 8.0;
                let sum_h = title_h + subtitle_h + gap;
                let y = row_h / 2.0 - sum_h / 2.0;

                nuon::label()
                    .y(y)
                    .text(self.title)
                    .text_justify(nuon::TextJustify::Left)
                    .font_size(title_h)
                    .size(row_inner_w, title_h)
                    .build(ui);
                nuon::label()
                    .y(y + gap + title_h)
                    .text(self.subtitle)
                    .color([1.0, 1.0, 1.0, 0.5])
                    .text_justify(nuon::TextJustify::Left)
                    .font_size(subtitle_h)
                    .y(row_h / 2.0)
                    .size(row_inner_w, subtitle_h)
                    .build(ui);
            }

            (self.body)(ui, row_inner_w, row_h);
        });
        nuon::translate().y(row_h).add_to_current(ui);
    }
}

pub fn settings_row<'a>() -> SettingsRow<'a> {
    SettingsRow::new()
}
