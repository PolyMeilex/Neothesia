use std::rc::Rc;

use iced_core::border::{Border, Radius};
use iced_graphics::core::Color;
use iced_style::{button, pick_list};

const SURFACE: Color = Color::from_rgb(
    0x30 as f32 / 255.0,
    0x34 as f32 / 255.0,
    0x3B as f32 / 255.0,
);

pub fn pick_list() -> iced_style::theme::PickList {
    iced_style::theme::PickList::Custom(Rc::new(PickListStyle), Rc::new(MenuStyle))
}

struct PickListStyle;

impl iced_style::pick_list::StyleSheet for PickListStyle {
    type Style = iced_style::Theme;

    fn active(&self, _style: &Self::Style) -> pick_list::Appearance {
        pick_list::Appearance {
            text_color: Color::WHITE,
            background: iced_core::Background::Color(Color::from_rgba8(74, 68, 88, 1.0)),
            placeholder_color: Color::WHITE,
            border: Border {
                radius: Radius::from(5.0),
                width: 0.0,
                color: SURFACE,
            },
            handle_color: Color::WHITE,
        }
    }

    fn hovered(&self, style: &Self::Style) -> pick_list::Appearance {
        let mut active = self.active(style);

        if let iced_core::Background::Color(ref mut color) = active.background {
            color.r = (color.r + 0.05).min(1.0);
            color.g = (color.g + 0.05).min(1.0);
            color.b = (color.b + 0.05).min(1.0);
        }

        active
    }
}

struct MenuStyle;

impl iced_style::menu::StyleSheet for MenuStyle {
    type Style = iced_style::Theme;

    fn appearance(&self, _style: &Self::Style) -> iced_style::menu::Appearance {
        let accent = Color::from_rgba8(160, 81, 255, 1.0);
        iced_style::menu::Appearance {
            text_color: Color::WHITE,
            background: iced_core::Background::from(iced_core::Color::from_rgba8(27, 25, 32, 1.0)),
            border: Border {
                width: 0.0,
                radius: Radius::from(5.0),
                color: SURFACE,
            },
            selected_text_color: Color::WHITE,
            selected_background: iced_core::Background::Color(accent),
        }
    }
}

pub fn button() -> iced_style::theme::Button {
    iced_style::theme::Button::Custom(Box::new(ButtonStyle))
}

struct ButtonStyle;

impl iced_style::button::StyleSheet for ButtonStyle {
    type Style = iced_style::Theme;

    fn active(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            text_color: Color::WHITE,
            border: Border {
                width: 0.0,
                radius: Radius::from(5.0),
                ..Default::default()
            },
            background: Some(iced_core::Background::Color(Color::from_rgba8(
                74, 68, 88, 1.0,
            ))),
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

pub fn round_button() -> iced_style::theme::Button {
    iced_style::theme::Button::Custom(Box::new(RoundButtonStyle))
}

struct RoundButtonStyle;

impl iced_style::button::StyleSheet for RoundButtonStyle {
    type Style = iced_style::Theme;

    fn active(&self, style: &Self::Style) -> button::Appearance {
        let def = ButtonStyle::active(&ButtonStyle, style);
        button::Appearance {
            border: Border {
                radius: Radius::from(f32::MAX),
                ..def.border
            },
            ..def
        }
    }

    fn hovered(&self, style: &Self::Style) -> button::Appearance {
        let def = ButtonStyle::hovered(&ButtonStyle, style);
        button::Appearance {
            border: Border {
                radius: Radius::from(f32::MAX),
                ..def.border
            },
            ..def
        }
    }
}

pub fn _checkbox() -> iced_style::theme::Checkbox {
    iced_style::theme::Checkbox::Custom(Box::new(CheckboxStyle))
}

struct CheckboxStyle;

impl iced_style::checkbox::StyleSheet for CheckboxStyle {
    type Style = iced_style::Theme;

    fn active(&self, _style: &Self::Style, is_checked: bool) -> iced_style::checkbox::Appearance {
        let active = Color::from_rgba8(160, 81, 255, 1.0);
        iced_style::checkbox::Appearance {
            background: if is_checked { active } else { SURFACE }.into(),
            text_color: Some(Color::WHITE),
            border: Border {
                radius: Radius::from(2.0),
                width: 1.0,
                color: active,
            },
            icon_color: Color::WHITE,
        }
    }

    fn hovered(&self, style: &Self::Style, is_checked: bool) -> iced_style::checkbox::Appearance {
        let active = Color::from_rgba8(160, 81, 255, 1.0);
        iced_style::checkbox::Appearance {
            background: Color {
                a: 0.8,
                ..if is_checked { active } else { SURFACE }
            }
            .into(),
            ..self.active(style, is_checked)
        }
    }

    fn disabled(&self, style: &Self::Style, is_checked: bool) -> iced_style::checkbox::Appearance {
        // TODO
        self.active(style, is_checked)
    }
}

pub fn toggler() -> iced_style::theme::Toggler {
    iced_style::theme::Toggler::Custom(Box::new(TogglerStyle))
}

struct TogglerStyle;

impl iced_style::toggler::StyleSheet for TogglerStyle {
    type Style = iced_style::Theme;

    fn active(&self, _style: &Self::Style, is_active: bool) -> iced_style::toggler::Appearance {
        let default = <iced_style::Theme as iced_style::toggler::StyleSheet>::active(
            &iced_style::Theme::Dark,
            &iced_style::theme::Toggler::Default,
            is_active,
        );

        if is_active {
            iced_style::toggler::Appearance {
                background: Color::from_rgba8(160, 81, 255, 1.0),
                ..default
            }
        } else {
            default
        }
    }

    fn hovered(&self, _style: &Self::Style, is_active: bool) -> iced_style::toggler::Appearance {
        if is_active {
            let default = <iced_style::Theme as iced_style::toggler::StyleSheet>::active(
                &iced_style::Theme::Dark,
                &iced_style::theme::Toggler::Default,
                is_active,
            );

            iced_style::toggler::Appearance {
                background: Color::from_rgba8(180, 101, 255, 1.0),
                ..default
            }
        } else {
            <iced_style::Theme as iced_style::toggler::StyleSheet>::hovered(
                &iced_style::Theme::Dark,
                &iced_style::theme::Toggler::Default,
                is_active,
            )
        }
    }
}
