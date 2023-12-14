use std::rc::Rc;

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
            background: iced_core::Background::Color(Color::BLACK),
            placeholder_color: Color::WHITE,
            border_radius: iced_core::BorderRadius::from(2.0),
            border_width: 1.0,
            border_color: SURFACE,
            handle_color: Color::WHITE,
        }
    }

    fn hovered(&self, _style: &Self::Style) -> pick_list::Appearance {
        let accent = Color::from_rgba8(160, 81, 255, 1.0);
        pick_list::Appearance {
            text_color: Color::WHITE,
            background: iced_core::Background::Color(Color::BLACK),
            // background: iced_graphics::Background::Color(Color::from_rgb8(42, 42, 42)),
            placeholder_color: Color::WHITE,
            border_radius: iced_core::BorderRadius::from(2.0),
            border_width: 1.0,
            // border_color: Color::from_rgb8(42, 42, 42),
            border_color: accent,
            handle_color: Color::WHITE,
        }
    }
}

struct MenuStyle;

impl iced_style::menu::StyleSheet for MenuStyle {
    type Style = iced_style::Theme;

    fn appearance(&self, _style: &Self::Style) -> iced_style::menu::Appearance {
        let accent = Color::from_rgba8(160, 81, 255, 1.0);
        iced_style::menu::Appearance {
            text_color: Color::WHITE,
            background: iced_core::Background::Color(Color::BLACK),
            border_width: 1.0,
            border_radius: iced_core::BorderRadius::from(0.0),
            border_color: SURFACE,
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
            border_color: SURFACE,
            border_width: 1.0,
            background: Some(iced_core::Background::Color(Color::BLACK)),
            ..Default::default()
        }
    }

    fn hovered(&self, _style: &Self::Style) -> button::Appearance {
        let accent = Color::from_rgba8(160, 81, 255, 1.0);
        button::Appearance {
            text_color: Color::WHITE,
            border_color: accent,
            border_width: 1.0,
            background: Some(iced_core::Background::Color(Color::BLACK)),
            ..Default::default()
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
            border_radius: iced_core::BorderRadius::from(2.0),
            border_width: 1.0,
            border_color: active,
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
}
