use std::usize;

use super::Element;
use iced_core::{image::Handle as ImageHandle, Alignment, Color};

mod theme;

fn get_instrument_icon(index: usize) -> ImageHandle {
    match index {
        797979 => ImageHandle::from_memory(
            include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/src/track_card/img/percussions.png"
            ))
            .to_vec(),
        ), // Percussions
        696969 => ImageHandle::from_memory(
            include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/src/track_card/img/piano-right.png"
            ))
            .to_vec(),
        ), // Pianos Right hand mark
        0..=7 => ImageHandle::from_memory(
            include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/src/track_card/img/piano-left.png"
            ))
            .to_vec(),
        ), // Pianos, default will be left hand
        8..=15 => ImageHandle::from_memory(
            include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/src/track_card/img/xylophones.png"
            ))
            .to_vec(),
        ), // Chromatic Percussion
        16..=23 => ImageHandle::from_memory(
            include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/src/track_card/img/brasses.png"
            ))
            .to_vec(),
        ), // Organs
        24..=31 => ImageHandle::from_memory(
            include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/src/track_card/img/guitars.png"
            ))
            .to_vec(),
        ), // Guitars
        32..=39 => ImageHandle::from_memory(
            include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/src/track_card/img/guitars.png"
            ))
            .to_vec(),
        ), // Basses
        40..=47 => ImageHandle::from_memory(
            include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/src/track_card/img/violins.png"
            ))
            .to_vec(),
        ), // Strings
        48..=51 => ImageHandle::from_memory(
            include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/src/track_card/img/uncategorized.png"
            ))
            .to_vec(),
        ), // Ensemble
        52..=55 => ImageHandle::from_memory(
            include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/src/track_card/img/choirs.png"
            ))
            .to_vec(),
        ), // choirs
        56..=63 => ImageHandle::from_memory(
            include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/src/track_card/img/brasses.png"
            ))
            .to_vec(),
        ), // Brass
        64..=71 => ImageHandle::from_memory(
            include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/src/track_card/img/flutes.png"
            ))
            .to_vec(),
        ), // Reeds
        72..=79 => ImageHandle::from_memory(
            include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/src/track_card/img/flutes.png"
            ))
            .to_vec(),
        ), // Pipes
        80..=87 => ImageHandle::from_memory(
            include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/src/track_card/img/uncategorized.png"
            ))
            .to_vec(),
        ), // Synth Leads
        88..=95 => ImageHandle::from_memory(
            include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/src/track_card/img/uncategorized.png"
            ))
            .to_vec(),
        ), // Synth Pads
        96..=103 => ImageHandle::from_memory(
            include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/src/track_card/img/uncategorized.png"
            ))
            .to_vec(),
        ), // Synth Effects
        104..=111 => ImageHandle::from_memory(
            include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/src/track_card/img/percussions.png"
            ))
            .to_vec(),
        ), // Ethnic
        112..=119 => ImageHandle::from_memory(
            include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/src/track_card/img/percussions.png"
            ))
            .to_vec(),
        ), // Percussive
        120..=127 => ImageHandle::from_memory(
            include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/src/track_card/img/uncategorized.png"
            ))
            .to_vec(),
        ), // Sound effects
        _ => ImageHandle::from_memory(
            include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/src/track_card/img/uncategorized.png"
            ))
            .to_vec(),
        ), // Default to Uncategorized
    }
}

pub struct TrackCard<'a, MSG> {
    title: String,
    subtitle: String,
    body: Option<Element<'a, MSG>>,
    track_color: Color,
    on_icon_press: Option<MSG>,
    instrument_id: usize,
}

impl<'a, MSG: 'a + Clone> Default for TrackCard<'a, MSG> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, MSG: 'a + Clone> TrackCard<'a, MSG> {
    pub fn new() -> Self {
        Self {
            title: String::new(),
            subtitle: String::new(),
            body: None,
            instrument_id: 0,

            track_color: Color::from_rgba8(210, 89, 222, 1.0),
            on_icon_press: None,
        }
    }

    pub fn title(mut self, text: impl ToString) -> Self {
        self.title = text.to_string();
        self
    }

    pub fn subtitle(mut self, text: impl ToString) -> Self {
        self.subtitle = text.to_string();
        self
    }

    pub fn instrument_id(mut self, instrument_id: usize) -> Self {
        self.instrument_id = instrument_id;
        self
    }

    pub fn track_color(mut self, color: Color) -> Self {
        self.track_color = color;
        self
    }

    pub fn on_icon_press(mut self, msg: MSG) -> Self {
        self.on_icon_press = Some(msg);
        self
    }

    pub fn body(mut self, body: impl Into<Element<'a, MSG>>) -> Self {
        self.body = Some(body.into());
        self
    }
}
impl<'a, M: Clone + 'a> From<TrackCard<'a, M>> for Element<'a, M> {
    fn from(card: TrackCard<'a, M>) -> Self {
        let header_content = vec![
            iced_widget::text(card.title).size(16).width(187).into(),
            iced_widget::text(card.subtitle).size(14).into(),
        ];

        let img = get_instrument_icon(card.instrument_id);

        let img_toggle = if card.track_color == iced_core::Color::from_rgb8(102, 102, 102) {
            ImageHandle::from_memory(include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/src/track_card/img/toggle_off.png"
            )))
        } else {
            ImageHandle::from_memory(include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/src/track_card/img/toggle_on.png"
            )))
        };

        let on_press_clone = card.on_icon_press.clone();

        let button1 = iced_widget::button(iced_widget::image(img).width(55))
            .width(55)
            .height(55)
            .style(theme::track_icon_button(iced_core::Color::TRANSPARENT))
            .on_press_maybe(card.on_icon_press);

        let button2 = iced_widget::button(iced_widget::image(img_toggle).width(40).height(15))
            .width(55)
            .height(15)
            .padding(0)
            .style(theme::toggle_button(iced_core::Color::TRANSPARENT))
            .on_press_maybe(on_press_clone);

        let header = iced_widget::row![
            button1,
            iced_widget::column(header_content)
                .spacing(0)
                .align_items(Alignment::Start),
            button2,
        ]
        .spacing(8);

        let mut children = vec![header.into()];
        if let Some(body) = card.body {
            children.push(body);
        }

        iced_widget::container(iced_widget::column(children).width(312).spacing(12))
            .padding(16)
            .style(theme::card())
            .into()
    }
}
