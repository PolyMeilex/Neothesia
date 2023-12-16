use super::Renderer;

static ICONS: iced_core::Font = iced_core::Font::with_name("bootstrap-icons");

pub fn play_icon<'a>() -> iced_widget::Text<'a, Renderer> {
    iced_widget::text('\u{f49d}').font(ICONS)
}

pub fn note_list_icon<'a>() -> iced_widget::Text<'a, Renderer> {
    iced_widget::text('\u{f451}').font(ICONS)
}

pub fn left_arrow_icon<'a>() -> iced_widget::Text<'a, Renderer> {
    iced_widget::text('\u{f12f}').font(ICONS)
}
