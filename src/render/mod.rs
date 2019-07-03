mod keyboard;
mod note;

use glium::Surface;

pub struct GameRenderer<'a> {
  display: &'a glium::Display,
  note_renderer: note::NoteRenderer<'a>,
  keyboard_renderer: keyboard::KeyboardRenderer<'a>,

  pub viewport: glium::Rect,
  pub update_size: bool,
  pub m_pos_x: f64,
  pub m_pos_y: f64,

  pub window_w: u32,
  pub window_h: u32,
}

impl<'a> GameRenderer<'a> {
  pub fn new(display: &'a glium::Display) -> GameRenderer<'a> {
    let viewport = glium::Rect {
      left: 0,
      bottom: 0,
      width: 1280,
      height: 720,
    };
    GameRenderer {
      display,
      viewport,
      note_renderer: note::NoteRenderer::new(display),
      keyboard_renderer: keyboard::KeyboardRenderer::new(display),
      update_size: true,
      m_pos_x: 0.0,
      m_pos_y: 0.0,
      window_w: 1,
      window_h: 1,
    }
  }
  pub fn draw(&mut self) {
    let mut target = self.display.draw();

    if self.update_size {
      let (width, height) = target.get_dimensions();
      self.window_w = width;
      self.window_h = height;

      let ar = 16.0 / 9.0;
      self.viewport.width = width;
      self.viewport.height = (width as f64 / ar) as u32;

      if height >= self.viewport.height {
        self.viewport.bottom = (height - self.viewport.height) / 2;
      }

      // No need to update it on every frame, when window has same size
      self.update_size = false;
    }

    target.clear_color_srgb(0.1, 0.1, 0.1, 1.0);
    self
      .keyboard_renderer
      .draw(&mut target, self, self.m_pos_x, self.m_pos_y);
    self
      .note_renderer
      .draw(&mut target, self, self.m_pos_x, self.m_pos_y);
    target.finish().unwrap();
  }
}

