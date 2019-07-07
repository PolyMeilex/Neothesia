use crate::game_states::GameState;

mod keyboard;
mod note;

pub struct PlayingState<'a> {
  display: &'a glium::Display,
  notes: Vec<crate::lib_midi::track::MidiNote>,
  keyboard: keyboard::KeyboardRenderer<'a>,
  note_renderer: Option<note::NoteRenderer<'a>>,
}

impl<'a> PlayingState<'a> {
  pub fn new(display: &'a glium::Display) -> Self {
    PlayingState {
      display,
      notes: Vec::new(),
      keyboard: keyboard::KeyboardRenderer::new(display),
      note_renderer: None,
    }
  }
}

use crate::game_states::StateUpdateMessage;

impl<'a> GameState<'a> for PlayingState<'a> {
  fn update(&mut self, msg: StateUpdateMessage) {
    match msg {
      StateUpdateMessage::PlayingState(notes) => {
        self.notes = notes;
        self.note_renderer = Some(note::NoteRenderer::new(self.display, &self.notes));
      }
      _ => panic!("This Message Should Not Reach PlayingState"),
    }
  }
  fn draw(&self, target: &mut glium::Frame, game_renderer: &crate::render::GameRenderer) {
    let time = game_renderer.time - 5.0;
    match &self.note_renderer {
      Some(note_renderer) => note_renderer.draw(target, game_renderer, time as f32),
      None => {}
    }

    let mut active_notes: [bool; 88] = [false; 88];

    for n in self.notes.iter() {
      if n.start < time + 3.0 && n.start - time > -0.1 {
        if n.start - time < 0.0 {
          active_notes[(n.note - 21) as usize] = true;
        }
      }
    }

    self.keyboard.draw(target, game_renderer, active_notes);
  }
}