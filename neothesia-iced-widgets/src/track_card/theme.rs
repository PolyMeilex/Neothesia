use iced_core::border::{Border, Radius};
use iced_widget::{button, container};

pub fn card<T>(_theme: &T) -> container::Style {
    container::Style {
        background: Some(iced_core::Background::from(iced_core::Color::from_rgba8(
            37, 35, 42, 1.0,
        ))),
        border: Border {
            radius: Radius::from(12.0),
            ..Default::default()
        },
        ..Default::default()
    }
}

pub fn track_icon_button<T>(
    color: iced_core::Color,
    theme: &T,
    status: button::Status,
) -> button::Style {
    match status {
        button::Status::Active => button::Style {
            background: Some(iced_core::Background::from(color)),
            border: Border {
                radius: Radius::from(255.0),
                ..Default::default()
            },
            ..Default::default()
        },
        button::Status::Hovered => {
            let mut active = track_icon_button(color, theme, button::Status::Active);

            if let Some(iced_core::Background::Color(ref mut color)) = active.background {
                color.r = (color.r + 0.05).min(1.0);
                color.g = (color.g + 0.05).min(1.0);
                color.b = (color.b + 0.05).min(1.0);
            }

            active
        }
        button::Status::Pressed => track_icon_button(color, theme, button::Status::Active),
        button::Status::Disabled => track_icon_button(color, theme, button::Status::Active),
    }
}
