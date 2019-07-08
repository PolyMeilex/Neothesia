use crate::game_states::GameState;

mod keyboard;
mod note;

use midir::MidiOutput;

pub struct PlayingState<'a> {
  display: &'a glium::Display,
  notes: Vec<lib_midi::track::MidiNote>,
  notes_on: Vec<lib_midi::track::MidiNote>,

  keyboard: keyboard::KeyboardRenderer<'a>,
  note_renderer: Option<note::NoteRenderer<'a>>,
  start_time: f64,
  midi_out: midir::MidiOutputConnection,
}

impl<'a> PlayingState<'a> {
  pub fn new(
    display: &'a glium::Display,
    notes: Vec<crate::lib_midi::track::MidiNote>,
    start_time: f64,
  ) -> Self {

    let midi_out = MidiOutput::new("midi").unwrap();
    let conn_out = midi_out.connect(1, "out").unwrap();

    let mut filtered_notes: Vec<crate::lib_midi::track::MidiNote> = Vec::new();
    for n in notes.iter() {
      if n.note > 21 && n.note < 109 {
        if n.ch != 9 {
          filtered_notes.push(n.clone());
        }
      }
    }

    let mut ps = PlayingState {
      display,
      notes: filtered_notes,
      keyboard: keyboard::KeyboardRenderer::new(display),
      note_renderer: None,
      start_time: start_time,
      midi_out: conn_out,
      notes_on: Vec::new(),
    };
    ps.note_renderer = Some(note::NoteRenderer::new(ps.display, &ps.notes));
    ps
  }
}

use crate::game_states::StateUpdateMessage;

impl<'a> GameState<'a> for PlayingState<'a> {
  fn update(&mut self, msg: StateUpdateMessage) {}
  fn draw(
    &mut self,
    target: &mut glium::Frame,
    public_state: &crate::render::PublicState,
  ) -> Option<Box<GameState<'a> + 'a>> {
    let time = public_state.time - self.start_time - 5.0;

    match &self.note_renderer {
      Some(note_renderer) => note_renderer.draw(target, &public_state.viewport, time as f32),
      None => {}
    }

    let mut active_notes: [bool; 88] = [false; 88];

    // for n in self.notes.iter() {
    //   if n.start < time + 3.0 && n.start - time > -0.1 {
    //     if n.start - time < 0.0 {
    //       active_notes[(n.note - 21) as usize] = true;
    //     }
    //   }
    // }
    let midi_out = &mut self.midi_out;
    self.notes_on.retain(|no| {
      let delete = {
        if time >= no.start + no.duration {
          midi_out.send(&[0x80, no.note, no.vel]).unwrap();
          true
        } else {
          false
        }
      };
      !delete
    });
    println!("N:{}", "=".repeat(self.notes_on.len()));

    for n in self.notes.iter() {
      if n.start < time && n.start - time > -0.1 {
        if n.start - time < 0.0 {
          active_notes[(n.note - 21) as usize] = true;
          self.notes_on.push(n.clone());
          midi_out.send(&[0x90, n.note, n.vel]);
        }
      }
    }

    self
      .keyboard
      .draw(target, &public_state.viewport, active_notes);
    None
  }
}