use iced_core::{
    border::{Border, Radius},
    Color,
};
use iced_widget::button;

pub enum ButtonSegmentKind {
    Start,
    Center,
    End,
}

pub fn segment_button<T>(
    kind: ButtonSegmentKind,
    active: bool,
    active_color: Color,
    theme: &T,
    status: iced_widget::button::Status,
) -> button::Style {
    match status {
        button::Status::Active => {
            let border_radius = match kind {
                ButtonSegmentKind::Start => Radius::from([255.0, 0.0, 0.0, 255.0]),
                ButtonSegmentKind::Center => Radius::from(0.0),
                ButtonSegmentKind::End => Radius::from([0.0, 255.0, 255.0, 0.0]),
            };

            let background = if active {
                Some(iced_core::Background::from(active_color))
            } else {
                Some(iced_core::Background::from(iced_core::Color::from_rgba8(
                    74, 68, 88, 1.0,
                )))
            };

            button::Style {
                text_color: Color::WHITE,
                border: Border {
                    color: Color::TRANSPARENT,
                    width: 0.0,
                    radius: border_radius,
                },
                background,
                ..Default::default()
            }
        }
        iced_widget::button::Status::Hovered => {
            let mut active =
                segment_button(kind, active, active_color, theme, button::Status::Active);

            if let Some(iced_core::Background::Color(ref mut color)) = active.background {
                color.r = (color.r + 0.05).min(1.0);
                color.g = (color.g + 0.05).min(1.0);
                color.b = (color.b + 0.05).min(1.0);
            }

            active
        }
        iced_widget::button::Status::Pressed => {
            segment_button(kind, active, active_color, theme, button::Status::Active)
        }
        iced_widget::button::Status::Disabled => {
            segment_button(kind, active, active_color, theme, button::Status::Active)
        }
    }
}
