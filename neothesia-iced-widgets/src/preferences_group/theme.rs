use iced_core::border::{Border, Radius};
use iced_widget::container;

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

pub fn separator<T>(_theme: &T) -> container::Style {
    container::Style {
        background: Some(iced_core::Background::from(iced_core::Color::from_rgba8(
            16, 16, 16, 1.0,
        ))),
        ..Default::default()
    }
}

pub fn subtitle<T>(_theme: &T) -> iced_widget::text::Style {
    iced_widget::text::Style {
        color: Some(iced_core::Color::from_rgba(1.0, 1.0, 1.0, 0.5)),
    }
}
