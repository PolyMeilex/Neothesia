use iced_core::{
    border::{Border, Radius},
    Color,
};
use iced_style::button;

pub enum ButtonSegmentKind {
    Start,
    Center,
    End,
}

pub fn segment_button(
    kind: ButtonSegmentKind,
    active: bool,
    active_color: Color,
) -> iced_style::theme::Button {
    iced_style::theme::Button::Custom(Box::new(SegmentButtonStyle(kind, active, active_color)))
}

struct SegmentButtonStyle(ButtonSegmentKind, bool, Color);

impl iced_style::button::StyleSheet for SegmentButtonStyle {
    type Style = iced_style::Theme;

    fn active(&self, _style: &Self::Style) -> button::Appearance {
        let border_radius = match self.0 {
            ButtonSegmentKind::Start => Radius::from([255.0, 0.0, 0.0, 255.0]),
            ButtonSegmentKind::Center => Radius::from(0.0),
            ButtonSegmentKind::End => Radius::from([0.0, 255.0, 255.0, 0.0]),
        };
        let active = self.1;

        let background = if active {
            Some(iced_core::Background::from(self.2))
        } else {
            Some(iced_core::Background::from(iced_core::Color::from_rgba8(
                74, 68, 88, 1.0,
            )))
        };

        button::Appearance {
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

    fn hovered(&self, style: &Self::Style) -> button::Appearance {
        let mut active = self.active(style);

        if let Some(iced_core::Background::Color(ref mut color)) = active.background {
            color.r = (color.r + 0.05).min(1.0);
            color.g = (color.g + 0.05).min(1.0);
            color.b = (color.b + 0.05).min(1.0);
        }

        active
    }
}
