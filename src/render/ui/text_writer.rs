extern crate glium_glyph;

use glium_glyph::glyph_brush;
use glium_glyph::glyph_brush::{rusttype::Font, Layout, Section};

pub struct TextWriter<'a> {
  display: &'a glium::Display,
  brush: glium_glyph::GlyphBrush<'a, 'a>,
}

impl<'a> TextWriter<'a> {
  pub fn new(display: &'a glium::Display) -> Self {
    let roboto: &[u8] = include_bytes!("../../../res/Roboto-Regular.ttf");
    let fonts = vec![Font::from_bytes(roboto).unwrap()];
    let glyph_brush = glium_glyph::GlyphBrush::new(display, fonts);

    TextWriter {
      display,
      brush: glyph_brush,
    }
  }
  pub fn add(&mut self, text: &str, x: f32, y: f32, centered: bool, color: Option<[f32; 4]>) {
    let color = match color {
      Some(col) => col,
      None => [1.0, 1.0, 1.0, 1.0],
    };

    let layout = if centered {
      Layout::default()
        .h_align(glyph_brush::HorizontalAlign::Center)
        .v_align(glyph_brush::VerticalAlign::Center)
    } else {
      Layout::default()
    };

    self.brush.queue(Section {
      text,
      color,
      screen_position: (x, y),
      scale: glium_glyph::glyph_brush::rusttype::Scale::uniform(26.0),
      layout,
      ..Section::default()
    });
  }
  pub fn draw(&mut self, target: &mut glium::Frame) {
    self.brush.draw_queued(self.display, target);
  }
}
