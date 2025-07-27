use crate::{self as nuon, TextJustify, Ui};

pub struct SettingsSection {
    label: String,
    width: f32,
}

impl SettingsSection {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            width: 0.0,
        }
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    pub fn build(
        self,
        ui: &mut Ui,
        build: impl FnOnce(&mut Ui, &dyn Fn(&mut Ui, SettingsRow<'_>), &dyn Fn(&mut Ui)),
    ) {
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
            build(
                ui,
                &|ui, row| {
                    row.width(self.width).build_inner(ui);
                },
                &spacer,
            );

            nuon::translate().x(self.width).add_to_current(ui);
        });

        nuon::translate().y(pos.y).add_to_current(ui);
    }
}

pub fn settings_section(label: impl Into<String>) -> SettingsSection {
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

    fn build_inner(self, ui: &mut Ui) {
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
                let gap = 5.0;
                let sum_h = title_h + subtitle_h + gap;
                let y = super::center_y(row_h, sum_h);

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
                    .size(row_inner_w, subtitle_h)
                    .build(ui);
            }

            (self.body)(ui, row_inner_w, row_h);
        });
        nuon::translate().y(row_h).add_to_current(ui);
    }

    pub fn build(self, ui: &mut Ui, add: &dyn Fn(&mut Ui, SettingsRow<'a>)) {
        add(ui, self);
    }
}

pub fn settings_row<'a>() -> SettingsRow<'a> {
    SettingsRow::new()
}

pub enum SettingsRowSpinResult {
    Plus,
    Minus,
    Idle,
}

pub struct SettingsRowSpin<'a> {
    row: SettingsRow<'a>,
    up_id: String,
    down_id: String,
}

impl<'a> SettingsRowSpin<'a> {
    pub fn new() -> Self {
        Self {
            row: settings_row(),
            up_id: String::new(),
            down_id: String::new(),
        }
    }

    pub fn title(mut self, label: impl Into<String>) -> Self {
        self.row = self.row.title(label);
        self
    }

    pub fn subtitle(mut self, label: impl Into<String>) -> Self {
        self.row = self.row.subtitle(label);
        self
    }

    pub fn plus_id(mut self, id: impl Into<String>) -> Self {
        self.up_id = id.into();
        self
    }

    pub fn minus_id(mut self, id: impl Into<String>) -> Self {
        self.down_id = id.into();
        self
    }

    pub fn build(
        self,
        ui: &mut Ui,
        add: &dyn Fn(&mut Ui, SettingsRow<'_>),
    ) -> SettingsRowSpinResult {
        fn button() -> nuon::Button {
            nuon::button()
                .color([74, 68, 88])
                .preseed_color([74, 68, 88])
                .hover_color([87, 81, 101])
                .border_radius([16.0; 4])
        }

        pub fn minus_icon() -> &'static str {
            "\u{F2EA}"
        }

        pub fn plus_icon() -> &'static str {
            "\u{F4FE}"
        }

        let mut res = SettingsRowSpinResult::Idle;

        self.row
            .body(|ui, row_w, row_h| {
                let w = 30.0;
                let h = 30.0;
                let gap = 10.0;

                nuon::translate().x(row_w - w).add_to_current(ui);

                if button()
                    .id(self.up_id)
                    .y(nuon::center_y(row_h, h))
                    .size(w, h)
                    .icon(plus_icon())
                    .build(ui)
                {
                    res = SettingsRowSpinResult::Plus;
                }

                nuon::translate().x(-w).add_to_current(ui);
                nuon::translate().x(-gap).add_to_current(ui);

                if button()
                    .id(self.down_id)
                    .y(nuon::center_y(row_h, h))
                    .size(w, h)
                    .icon(minus_icon())
                    .build(ui)
                {
                    res = SettingsRowSpinResult::Minus;
                }
            })
            .build(ui, add);

        res
    }
}

pub fn settings_row_spin<'a>() -> SettingsRowSpin<'a> {
    SettingsRowSpin::new()
}
