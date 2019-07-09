use crate::game_states::GameState;
use std::collections::HashMap;

mod keyboard;
mod note;

use midir::MidiOutput;

pub struct PlayingState<'a> {
  display: &'a glium::Display,
  notes: Vec<lib_midi::track::MidiNote>,
  notes_on: HashMap<usize, bool>,

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

    let mut song_start_time = 0.0;
    if filtered_notes.len() > 0 {
      song_start_time = filtered_notes[0].start;
    }


    let mut ps = PlayingState {
      display,
      notes: filtered_notes,
      keyboard: keyboard::KeyboardRenderer::new(display),
      note_renderer: None,
      start_time: start_time - song_start_time + 5.0,
      midi_out: conn_out,
      notes_on: HashMap::new(),
    };
    ps.note_renderer = Some(note::NoteRenderer::new(ps.display, &ps.notes));
    ps
  }
}

impl<'a> GameState<'a> for PlayingState<'a> {
  fn draw(
    &mut self,
    target: &mut glium::Frame,
    public_state: &crate::render::PublicState,
  ) -> Option<Box<GameState<'a> + 'a>> {
    let time = public_state.time - self.start_time;

    match &self.note_renderer {
      Some(note_renderer) => note_renderer.draw(target, &public_state.viewport, time as f32),
      None => {}
    }

    let mut active_notes: [bool; 88] = [false; 88];

    let midi_out = &mut self.midi_out;
    let notes_on = &mut self.notes_on;

    self.notes.retain(|n| {
      if n.start <= time {
        if n.start + n.duration >= time {
          active_notes[(n.note - 21) as usize] = true;
          if !notes_on.contains_key(&n.id) {
            notes_on.insert(n.id, true);
            midi_out.send(&[0x90, n.note, n.vel]).ok();
          }
        } else {
          if notes_on.contains_key(&n.id) {
            notes_on.remove(&n.id);
            midi_out.send(&[0x80, n.note, n.vel]).ok();
          }
          // No need to keep note in vec after it was played
          return false;
        }
      }
      return true;
    });
    println!("Left:{}", self.notes.len());

    if self.notes.len() == 0 {
      let menu = Box::new(crate::game_states::MenuState::new(self.display));
      return Some(menu);
    }

    self
      .keyboard
      .draw(target, &public_state.viewport, active_notes);
    None
  }
}