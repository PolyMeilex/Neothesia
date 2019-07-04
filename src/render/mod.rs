mod keyboard;
mod note;

use glium::Surface;

pub struct GameRenderer<'a> {
  display: &'a glium::Display,
  note_renderer: Option<note::NoteRenderer<'a>>,
  keyboard_renderer: keyboard::KeyboardRenderer<'a>,
  notes: Vec<crate::lib_midi::track::MidiNote>,

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
      note_renderer: None,
      keyboard_renderer: keyboard::KeyboardRenderer::new(display),
      notes: Vec::new(),

      update_size: true,
      m_pos_x: 0.0,
      m_pos_y: 0.0,
      window_w: 1,
      window_h: 1,
    }
  }
  pub fn load_song(&mut self, track: crate::lib_midi::track::MidiTrack) {
    let mut notes: Vec<crate::lib_midi::track::MidiNote> = Vec::new();

    for n in track.notes.iter() {
      if n.note > 21 && n.note < 109 {
        if n.ch != 9 {
          notes.push(n.clone());
        }
      }
    }

    self.notes = notes;
    self.note_renderer = Some(note::NoteRenderer::new(self.display, &self.notes));
  }
  pub fn draw(&mut self, time: u128) {
    let time = time as f64 / 1000.0;

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

    match &self.note_renderer {
      Some(note_renderer) => note_renderer.draw(&mut target, self, time as f32),
      None => {}
    }


    let mut active_notes: [bool; 88] = [false; 88];

    // Causes a lot of lag when plaing Black Midi;
    for n in self.notes.iter() {
      if n.start < time + 3.0 && n.start - time > -0.1 {
        if n.start - time < 0.0 {
          active_notes[(n.note - 21) as usize] = true;
        }
      }
    }

    self.keyboard_renderer.draw(&mut target, self, active_notes);

    target.finish().unwrap();
  }
}

