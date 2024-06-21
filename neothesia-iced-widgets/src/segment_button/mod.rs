use super::{Element, Renderer};
use iced_core::{
    alignment::{Horizontal, Vertical},
    Color, Length, Theme,
};

mod theme;

pub struct SegmentButton<MSG> {
    buttons: Vec<(String, MSG)>,
    active_color: Color,
    active: usize,
}

impl<M> Default for SegmentButton<M> {
    fn default() -> Self {
        Self::new()
    }
}

impl<MSG> SegmentButton<MSG> {
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
}

fn segment<'a, MSG: 'a>(label: &str) -> iced_widget::Button<'a, MSG, Theme, Renderer> {
    iced_widget::button(
        iced_widget::text(label.to_string())
            .horizontal_alignment(Horizontal::Center)
            .vertical_alignment(Vertical::Center)
            .width(Length::Fill)
            .height(Length::Fill),
    )
    .padding(0)
    .width(Length::Fill)
    .height(Length::Fill)
}

impl<'a, M: Clone + 'a> From<SegmentButton<M>> for Element<'a, M> {
    fn from(mut btn: SegmentButton<M>) -> Self {
        let mut new = Vec::new();

        let last_id = btn.buttons.len() - 1;

        let first = btn.buttons.remove(0);
        let first = segment(&first.0)
            .style({
                let active = btn.active == 0;
                let active_color = btn.active_color;
                move |theme, status| {
                    theme::segment_button(
                        theme::ButtonSegmentKind::Start,
                        active,
                        active_color,
                        theme,
                        status,
                    )
                }
            })
            .on_press(first.1);

        let last = btn.buttons.pop().unwrap();
        let last = segment(&last.0)
            .style({
                let active = btn.active == last_id;
                let active_color = btn.active_color;
                move |theme, status| {
                    theme::segment_button(
                        theme::ButtonSegmentKind::End,
                        active,
                        active_color,
                        theme,
                        status,
                    )
                }
            })
            .on_press(last.1);

        new.push(first);
        for (id, (label, msg)) in btn.buttons.into_iter().enumerate() {
            let id = id + 1;
            new.push(
                segment(&label)
                    .style({
                        let active = btn.active == id;
                        let active_color = btn.active_color;
                        move |theme, status| {
                            theme::segment_button(
                                theme::ButtonSegmentKind::Center,
                                active,
                                active_color,
                                theme,
                                status,
                            )
                        }
                    })
                    .on_press(msg),
            );
        }
        new.push(last);

        let new: Vec<Element<M>> = new.into_iter().map(|btn| btn.into()).collect();
        iced_widget::container(iced_widget::row(new))
            .height(40)
            .width(Length::Fill)
            .into()
    }
}
