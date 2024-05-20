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
                44, 59, 102, 1.0,
            ))),
            border: Border {
                radius: Radius::from(0),

                ..Default::default()
            },
            ..Default::default()
        }
    }
}
