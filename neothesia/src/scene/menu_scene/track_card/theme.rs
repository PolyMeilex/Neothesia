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

pub fn track_icon_button(color: iced_core::Color) -> iced_style::theme::Button {
    iced_style::theme::Button::Custom(Box::new(TrackIconButtonStyle(color)))
}

struct TrackIconButtonStyle(iced_core::Color);

impl iced_style::button::StyleSheet for TrackIconButtonStyle {
    type Style = iced_style::Theme;

    fn active(&self, _style: &Self::Style) -> iced_style::button::Appearance {
        iced_style::button::Appearance {
            background: Some(iced_core::Background::from(self.0)),
            border_radius: BorderRadius::from(255.0),
            ..Default::default()
        }
    }

    /// Produces the hovered [`Appearance`] of a button.
    fn hovered(&self, style: &Self::Style) -> iced_style::button::Appearance {
        let mut active = self.active(style);

        if let Some(iced_core::Background::Color(ref mut color)) = active.background {
            color.r = (color.r + 0.05).min(1.0);
            color.g = (color.g + 0.05).min(1.0);
            color.b = (color.b + 0.05).min(1.0);
        }

        active
    }
}
