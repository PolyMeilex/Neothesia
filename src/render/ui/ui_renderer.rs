use crate::render::ui::button::ButtonsRenderer;
use crate::render::ui::text_writer::TextWriter;

pub struct UiRenderer<'a> {
  pub text_writer: TextWriter<'a>,
  pub buttons_renderer: ButtonsRenderer,
}

impl<'a> UiRenderer<'a> {
  pub fn new(display: &'a glium::Display) -> Self {
    Self {
      text_writer: TextWriter::new(display),
      buttons_renderer: ButtonsRenderer::new(display),
    }
  }
  pub fn draw(&mut self, target: &mut glium::Frame) {
    self.text_writer.draw(target);
  }
}