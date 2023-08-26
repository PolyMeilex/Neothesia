use super::Renderer;
use iced_core::{Alignment, Color, Element};

mod theme;

pub struct TrackCard<'a, MSG> {
    title: String,
    subtitle: String,
    body: Option<Element<'a, MSG, Renderer>>,
    track_color: Color,
}

impl<'a, MSG: 'a> TrackCard<'a, MSG> {
    pub fn new() -> Self {
        Self {
            title: String::new(),
            subtitle: String::new(),
            body: None,
            track_color: Color::from_rgba8(210, 89, 222, 1.0),
        }
    }

    pub fn title(mut self, text: impl ToString) -> Self {
        self.title = text.to_string();
        self
    }

    pub fn subtitle(mut self, text: impl ToString) -> Self {
        self.subtitle = text.to_string();
        self
    }

    pub fn track_color(mut self, color: Color) -> Self {
        self.track_color = color;
        self
    }

    pub fn body(mut self, body: impl Into<Element<'a, MSG, Renderer>>) -> Self {
        self.body = Some(body.into());
        self
    }

    pub fn build(self) -> iced_widget::Container<'a, MSG, Renderer> {
        let header = {
            iced_widget::row![
                iced_widget::container(iced_widget::text(""))
                    .width(40)
                    .height(40)
                    .style(theme::track_icon(self.track_color)),
                iced_widget::column(vec![
                    iced_widget::text(self.title).size(16).into(),
                    iced_widget::text(self.subtitle).size(14).into(),
                ])
                .spacing(4)
                .align_items(Alignment::Start),
            ]
            .spacing(16)
        };

        let mut children = vec![header.into()];
        if let Some(body) = self.body {
            children.push(body);
        }

        iced_widget::container(iced_widget::column(children).width(312).spacing(12))
            .padding(16)
            .style(theme::card())
    }
}

pub fn track_card<'a, MSG: 'a>() -> TrackCard<'a, MSG> {
    TrackCard::new()
}
