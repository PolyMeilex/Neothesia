use super::Renderer;
use iced_core::{
    alignment::{Horizontal, Vertical},
    Color, Length,
};

mod theme;

fn segment<'a, MSG: 'a>(label: &str) -> iced_widget::Button<'a, MSG, Renderer> {
    iced_widget::button(
        iced_widget::text(label)
            .horizontal_alignment(Horizontal::Center)
            .vertical_alignment(Vertical::Center)
            .width(Length::Fill)
            .height(Length::Fill),
    )
    .padding(0)
    .width(Length::Fill)
    .height(Length::Fill)
}

pub fn segment_button<MSG: Clone>() -> SegmentButton<MSG> {
    SegmentButton::new()
}

pub struct SegmentButton<MSG> {
    buttons: Vec<(String, MSG)>,
    active_color: Color,
    active: usize,
}

impl<'a, MSG: Clone + 'a> SegmentButton<MSG> {
    pub fn new() -> Self {
        Self {
            buttons: vec![],
            active: 0,
            active_color: Color::from_rgba8(210, 89, 222, 1.0),
        }
    }

    pub fn active(mut self, active: usize) -> Self {
        self.active = active;
        self
    }

    pub fn button(mut self, label: &str, on_press: MSG) -> Self {
        self.buttons.push((label.to_string(), on_press));
        self
    }

    pub fn active_color(mut self, color: Color) -> Self {
        self.active_color = color;
        self
    }

    pub fn build(mut self) -> iced_widget::Container<'a, MSG, Renderer> {
        let mut new = Vec::new();

        let last_id = self.buttons.len() - 1;

        let first = self.buttons.remove(0);
        let first = segment(&first.0)
            .style(theme::segment_button(
                theme::ButtonSegmentKind::Start,
                self.active == 0,
                self.active_color,
            ))
            .on_press(first.1);

        let last = self.buttons.pop().unwrap();
        let last = segment(&last.0)
            .style(theme::segment_button(
                theme::ButtonSegmentKind::End,
                self.active == last_id,
                self.active_color,
            ))
            .on_press(last.1);

        new.push(first);
        for (id, (label, msg)) in self.buttons.into_iter().enumerate() {
            let id = id + 1;
            new.push(
                segment(&label)
                    .style(theme::segment_button(
                        theme::ButtonSegmentKind::Center,
                        self.active == id,
                        self.active_color,
                    ))
                    .on_press(msg),
            );
        }
        new.push(last);

        iced_widget::container(iced_widget::row(
            new.into_iter().map(|btn| btn.into()).collect(),
        ))
        .height(40)
        .width(Length::Fill)
    }
}
