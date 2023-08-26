use iced_core::BorderRadius;

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
            border_radius: BorderRadius::from(12.0),
            ..Default::default()
        }
    }
}

pub fn track_icon(color: iced_core::Color) -> iced_style::theme::Container {
    iced_style::theme::Container::Custom(Box::new(TrackIconStyle(color)))
}

struct TrackIconStyle(iced_core::Color);

impl iced_style::container::StyleSheet for TrackIconStyle {
    type Style = iced_style::Theme;

    fn appearance(&self, _style: &Self::Style) -> iced_style::container::Appearance {
        iced_style::container::Appearance {
            background: Some(iced_core::Background::from(self.0)),
            border_radius: BorderRadius::from(255.0),
            ..Default::default()
        }
    }
}
