use iced_core::{
    border::{Border, Radius},
    Theme,
};
use iced_graphics::core::Color;

const SURFACE: Color = Color::from_rgb(
    0x30 as f32 / 255.0,
    0x34 as f32 / 255.0,
    0x3B as f32 / 255.0,
);

pub fn pick_list(
    _theme: &Theme,
    status: iced_widget::pick_list::Status,
) -> iced_widget::pick_list::Style {
    match status {
        iced_widget::pick_list::Status::Active => iced_widget::pick_list::Style {
            text_color: Color::WHITE,
            placeholder_color: Color::WHITE,
            handle_color: Color::WHITE,
            background: iced_core::Background::Color(Color::from_rgba8(74, 68, 88, 1.0)),
            border: Border {
                radius: Radius::from(5.0),
                width: 0.0,
                color: SURFACE,
            },
        },
        iced_widget::pick_list::Status::Hovered => {
            let mut active = pick_list(_theme, iced_widget::pick_list::Status::Active);

            if let iced_core::Background::Color(ref mut color) = active.background {
                color.r = (color.r + 0.05).min(1.0);
                color.g = (color.g + 0.05).min(1.0);
                color.b = (color.b + 0.05).min(1.0);
            }

            active
        }
        iced_widget::pick_list::Status::Opened { .. } => {
            pick_list(_theme, iced_widget::pick_list::Status::Active)
        }
    }
}

pub fn pick_list_menu(_theme: &Theme) -> iced_widget::overlay::menu::Style {
    let accent = Color::from_rgba8(160, 81, 255, 1.0);
    iced_widget::overlay::menu::Style {
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

pub fn button(_theme: &Theme, status: iced_widget::button::Status) -> iced_widget::button::Style {
    match status {
        iced_widget::button::Status::Active => iced_widget::button::Style {
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
        },
        iced_widget::button::Status::Hovered => {
            let mut active = button(_theme, iced_widget::button::Status::Active);

            if let Some(iced_core::Background::Color(ref mut color)) = active.background {
                color.r = (color.r + 0.05).min(1.0);
                color.g = (color.g + 0.05).min(1.0);
                color.b = (color.b + 0.05).min(1.0);
            }

            active
        }
        _ => button(_theme, iced_widget::button::Status::Active),
    }
}

pub fn round_button(
    theme: &Theme,
    status: iced_widget::button::Status,
) -> iced_widget::button::Style {
    match status {
        iced_widget::button::Status::Active => {
            let def = button(theme, status);
            iced_widget::button::Style {
                border: Border {
                    radius: Radius::from(f32::MAX),
                    ..def.border
                },
                ..def
            }
        }
        iced_widget::button::Status::Hovered => {
            let def = button(theme, status);
            iced_widget::button::Style {
                border: Border {
                    radius: Radius::from(f32::MAX),
                    ..def.border
                },
                ..def
            }
        }
        _ => round_button(theme, iced_widget::button::Status::Active),
    }
}

pub fn toggler(theme: &Theme, status: iced_widget::toggler::Status) -> iced_widget::toggler::Style {
    match status {
        iced_widget::toggler::Status::Active { is_toggled } => {
            let default = iced_widget::toggler::default(theme, status);

            if is_toggled {
                iced_widget::toggler::Style {
                    background: Color::from_rgba8(160, 81, 255, 1.0),
                    ..default
                }
            } else {
                default
            }
        }
        iced_widget::toggler::Status::Hovered { is_toggled } => {
            if is_toggled {
                let default = iced_widget::toggler::default(theme, status);
                iced_widget::toggler::Style {
                    background: Color::from_rgba8(180, 101, 255, 1.0),
                    ..default
                }
            } else {
                iced_widget::toggler::default(theme, status)
            }
        }
        iced_widget::toggler::Status::Disabled => iced_widget::toggler::default(theme, status),
    }
}

pub fn scrollable(
    _theme: &Theme,
    status: iced_widget::scrollable::Status,
) -> iced_widget::scrollable::Style {
    match status {
        iced_widget::scrollable::Status::Active {
            is_horizontal_scrollbar_disabled,
            is_vertical_scrollbar_disabled,
        } => scrollable(
            _theme,
            iced_widget::scrollable::Status::Hovered {
                is_horizontal_scrollbar_hovered: false,
                is_vertical_scrollbar_hovered: false,
                is_horizontal_scrollbar_disabled,
                is_vertical_scrollbar_disabled,
            },
        ),
        iced_widget::scrollable::Status::Hovered {
            is_horizontal_scrollbar_hovered,
            is_vertical_scrollbar_hovered,
            is_horizontal_scrollbar_disabled: _,
            is_vertical_scrollbar_disabled: _,
        } => iced_widget::scrollable::Style {
            container: iced_widget::container::Style::default(),
            vertical_rail: iced_widget::scrollable::Rail {
                background: Some(iced_core::Background::Color(Color::from_rgba8(
                    37, 35, 42, 1.0,
                ))),
                border: Border::default().rounded(10.0),
                scroller: iced_widget::scrollable::Scroller {
                    color: if is_vertical_scrollbar_hovered {
                        Color::from_rgba8(87, 81, 101, 1.0)
                    } else {
                        Color::from_rgba8(74, 68, 88, 1.0)
                    },
                    border: Border::default().rounded(10.0),
                },
            },
            horizontal_rail: iced_widget::scrollable::Rail {
                background: Some(iced_core::Background::Color(Color::from_rgba8(
                    37, 35, 42, 1.0,
                ))),
                border: Border::default().rounded(10.0),
                scroller: iced_widget::scrollable::Scroller {
                    color: if is_horizontal_scrollbar_hovered {
                        Color::from_rgba8(87, 81, 101, 1.0)
                    } else {
                        Color::from_rgba8(74, 68, 88, 1.0)
                    },
                    border: Border::default().rounded(10.0),
                },
            },
            gap: None,
        },
        iced_widget::scrollable::Status::Dragged {
            is_horizontal_scrollbar_dragged,
            is_vertical_scrollbar_dragged,
            is_horizontal_scrollbar_disabled,
            is_vertical_scrollbar_disabled,
        } => scrollable(
            _theme,
            iced_widget::scrollable::Status::Hovered {
                is_horizontal_scrollbar_hovered: is_horizontal_scrollbar_dragged,
                is_vertical_scrollbar_hovered: is_vertical_scrollbar_dragged,
                is_horizontal_scrollbar_disabled,
                is_vertical_scrollbar_disabled,
            },
        ),
    }
}
