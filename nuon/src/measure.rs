use std::{
    borrow::Cow,
    collections::HashMap,
    hash::{DefaultHasher, Hash, Hasher},
};

use cosmic_text::{Attrs, Buffer, Family, Metrics, Shaping, Weight};

use crate::{Point, Rect, Size, TextAlign, TextStyle, font_system::font_system};

#[derive(Debug, Clone)]
pub struct TextLayout {
    pub text: Cow<'static, str>,
    pub size: f32,
    pub font_family: Cow<'static, str>,
    pub bold: bool,
    pub bounds: Rect,
    pub text_justify: TextAlign,
    pub text_align: TextAlign,
    pub styles: Vec<TextStyle>,
}

impl TextLayout {
    pub fn new(text: impl Into<Cow<'static, str>>) -> Self {
        Self {
            text: text.into(),
            size: 16.0,
            font_family: Cow::Borrowed("Roboto"),
            bold: false,
            bounds: Rect::zero(),
            text_justify: TextAlign::Start,
            text_align: TextAlign::Start,
            styles: Vec::new(),
        }
    }

    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    pub fn font_family(mut self, font_family: impl Into<Cow<'static, str>>) -> Self {
        self.font_family = font_family.into();
        self
    }

    pub fn bold(mut self, bold: bool) -> Self {
        self.bold = bold;
        self
    }

    pub fn bounds(mut self, bounds: Rect) -> Self {
        self.bounds = bounds;
        self
    }

    pub fn text_justify(mut self, text_justify: TextAlign) -> Self {
        self.text_justify = text_justify;
        self
    }

    pub fn text_align(mut self, text_align: TextAlign) -> Self {
        self.text_align = text_align;
        self
    }

    pub fn styles(mut self, styles: Vec<TextStyle>) -> Self {
        self.styles = styles;
        self
    }

    fn shape(&self) -> Buffer {
        let font_system = font_system();
        let font_system = &mut font_system.borrow_mut();

        let mut attrs = Attrs::new().family(Family::Name(&self.font_family));
        if self.bold {
            attrs = attrs.weight(Weight::BOLD);
        }

        let mut buffer = Buffer::new(font_system, Metrics::new(self.size, self.size));
        buffer.set_size(Some(f32::MAX), Some(f32::MAX));
        buffer.set_rich_text(self.spans(&attrs), &attrs, Shaping::Basic, None);
        buffer.shape_until_scroll(font_system, false);
        buffer
    }

    fn spans<'a>(&'a self, attrs: &Attrs<'a>) -> Vec<(&'a str, Attrs<'a>)> {
        let text = self.text.as_ref();
        let mut spans = Vec::new();
        let mut pos = 0;
        for style in self.styles.iter() {
            let range = &style.range;
            let start = range.start.max(pos);
            let (Some(gap), Some(styled)) = (text.get(pos..start), text.get(start..range.end))
            else {
                continue;
            };
            spans.push((gap, attrs.clone()));
            spans.push((
                styled,
                attrs
                    .clone()
                    .color(cosmic_text::Color(style.color.packet_u32())),
            ));
            pos = range.end;
        }
        spans.push((&text[pos..], attrs.clone()));
        spans
    }

    pub(super) fn place(&self, text_size: Size) -> Rect {
        fn align(align: TextAlign, start: f32, container: f32, item: f32) -> f32 {
            match align {
                TextAlign::Start => start,
                TextAlign::Center => start + container / 2.0 - item / 2.0,
                TextAlign::End => start + container - item,
            }
        }

        let origin = self.bounds.origin;
        let bounds = self.bounds.size;

        Rect::new(
            Point::new(
                align(self.text_justify, origin.x, bounds.width, text_size.width),
                align(self.text_align, origin.y, bounds.height, text_size.height),
            ),
            text_size,
        )
    }

    fn cache_key(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        (
            &self.text,
            self.size.to_bits(),
            &self.font_family,
            self.bold,
            &self.styles,
        )
            .hash(&mut hasher);
        hasher.finish()
    }
}

fn buffer_size(buffer: &Buffer) -> Size {
    let (width, height) = buffer
        .layout_runs()
        .fold((0.0, 0.0), |(width, height), run| {
            (run.line_w.max(width), height + run.line_height)
        });

    Size::new(width, height)
}

struct CacheSlot {
    buffer: Buffer,
    size: Size,
    alive: bool,
}

#[derive(Default)]
pub struct TextCache {
    map: HashMap<u64, CacheSlot>,
}

impl TextCache {
    fn slot(&mut self, layout: &TextLayout) -> &CacheSlot {
        let slot = self.map.entry(layout.cache_key()).or_insert_with(|| {
            let buffer = layout.shape();
            CacheSlot {
                size: buffer_size(&buffer),
                buffer,
                alive: true,
            }
        });
        slot.alive = true;
        slot
    }

    pub fn buffer(&mut self, layout: &TextLayout) -> &Buffer {
        &self.slot(layout).buffer
    }

    pub fn measure(&mut self, layout: &TextLayout) -> Size {
        self.slot(layout).size
    }

    pub fn get(&self, layout: &TextLayout) -> Option<(&Buffer, Size)> {
        self.map
            .get(&layout.cache_key())
            .map(|slot| (&slot.buffer, slot.size))
    }

    pub fn frame_done(&mut self) {
        self.map.retain(|_, slot| std::mem::take(&mut slot.alive));
    }
}
