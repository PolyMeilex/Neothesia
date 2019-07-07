extern crate glium_glyph;

use glium_glyph::glyph_brush::{rusttype::Font, Section};


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
  pub fn add(&mut self, text: &str, x: f32, y: f32) {
    self.brush.queue(Section {
      text: text,
      color: [1.0, 1.0, 1.0, 1.0],
      screen_position: (x, y),
      scale: glium_glyph::glyph_brush::rusttype::Scale::uniform(26.0),
      ..Section::default()
    });
  }
  pub fn draw(&mut self, target: &mut glium::Frame) {
    self.brush.draw_queued(self.display, target);
  }
}

