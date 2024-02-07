use iced_core::border::{Border, Radius};

pub fn card() -> iced_style::theme::Container {
    iced_style::theme::Container::Custom(Box::new(ContainerStyle))
}

struct ContainerStyle;

impl iced_style::container::StyleSheet for ContainerStyle {
    type Style = iced_style::Theme;

    fn appearance(&self, _style: &Self::Style) -> iced_style::container::Appearance {
        iced_style::container::Appearance {
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
}

pub fn separator() -> iced_style::theme::Container {
    iced_style::theme::Container::Custom(Box::new(SeparatorContainerStyle))
}

struct SeparatorContainerStyle;

impl iced_style::container::StyleSheet for SeparatorContainerStyle {
    type Style = iced_style::Theme;

    fn appearance(&self, _style: &Self::Style) -> iced_style::container::Appearance {
        iced_style::container::Appearance {
            background: Some(iced_core::Background::from(iced_core::Color::from_rgba8(
                16, 16, 16, 1.0,
            ))),
            ..Default::default()
        }
    }
}

pub fn subtitle() -> iced_style::theme::Text {
    iced_style::theme::Text::Color(iced_core::Color::from_rgba(1.0, 1.0, 1.0, 0.5))
}
