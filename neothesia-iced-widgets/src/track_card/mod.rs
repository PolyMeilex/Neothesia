use super::Element;
use iced_core::{Alignment, Color};

mod theme;

pub struct TrackCard<'a, MSG> {
    title: String,
    subtitle: String,
    body: Option<Element<'a, MSG>>,
    track_color: Color,
    on_icon_press: Option<MSG>,
}

impl<'a, MSG: 'a + Clone> Default for TrackCard<'a, MSG> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, MSG: 'a + Clone> TrackCard<'a, MSG> {
    pub fn new() -> Self {
        Self {
            title: String::new(),
            subtitle: String::new(),
            body: None,
            track_color: Color::from_rgba8(210, 89, 222, 1.0),
            on_icon_press: None,
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

    pub fn on_icon_press(mut self, msg: MSG) -> Self {
        self.on_icon_press = Some(msg);
        self
    }

    pub fn body(mut self, body: impl Into<Element<'a, MSG>>) -> Self {
        self.body = Some(body.into());
        self
    }
}

impl<'a, M: Clone + 'a> From<TrackCard<'a, M>> for Element<'a, M> {
    fn from(card: TrackCard<'a, M>) -> Self {
        let header = {
            iced_widget::row![
                iced_widget::button(iced_widget::text(""))
                    .width(40)
                    .height(40)
                    .style({
                        let color = card.track_color;
                        move |theme, status| theme::track_icon_button(color, theme, status)
                    })
                    .on_press_maybe(card.on_icon_press),
                iced_widget::column(vec![
                    iced_widget::text(card.title).size(16).into(),
                    iced_widget::text(card.subtitle).size(14).into(),
                ])
                .spacing(4)
                .align_items(Alignment::Start),
            ]
            .spacing(16)
        };

        let mut children = vec![header.into()];
        if let Some(body) = card.body {
            children.push(body);
        }

        iced_widget::container(iced_widget::column(children).width(312).spacing(12))
            .padding(16)
            .style(theme::card)
            .into()
    }
}
