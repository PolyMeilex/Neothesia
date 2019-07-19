use crate::game_states::GameState;
use crate::game_states::GameStateType;
use std::collections::HashMap;

mod keyboard;
mod note;

pub struct PlayingState<'a> {
  state_type: GameStateType,
  display: &'a glium::Display,
  notes: Vec<lib_midi::track::MidiNote>,
  notes_on: HashMap<usize, bool>,

  keyboard: keyboard::KeyboardRenderer<'a>,
  note_renderer: Option<note::NoteRenderer>,
  start_time: f64,
  last_time: f32,
}

impl<'a> PlayingState<'a> {
  pub fn new(
    display: &'a glium::Display,
    notes: Vec<crate::lib_midi::track::MidiNote>,
    start_time: f64,
  ) -> Self {

    let mut filtered_notes: Vec<crate::lib_midi::track::MidiNote> = Vec::new();
    for n in notes.iter() {
      if n.note > 21 && n.note < 109 {
        if n.ch != 9 {
          filtered_notes.push(n.clone());
        }
      }
    }

    let song_start_time = if !filtered_notes.is_empty() {
      filtered_notes[0].start
    } else {
      0.0
    };

    let mut ps = PlayingState {
      state_type: GameStateType::PlayingState,
      display,
      notes: filtered_notes,
      keyboard: keyboard::KeyboardRenderer::new(display),
      note_renderer: None,
      start_time: start_time - f64::from(song_start_time) + 5.0,
      last_time: 0.0,
      notes_on: HashMap::new(),
    };
    ps.note_renderer = Some(note::NoteRenderer::new(ps.display, &ps.notes));
    ps
  }
}

impl<'a> GameState<'a> for PlayingState<'a> {
  fn get_type(&self) -> GameStateType {
    self.state_type
  }
  fn draw(
    &mut self,
    target: &mut glium::Frame,
    public_state: &mut crate::render::PublicState,
  ) -> Option<Box<dyn GameState<'a> + 'a>> {
    let time: f32 = (public_state.time - self.start_time) as f32;

    let midi_out = &mut public_state.midi_device;
    let notes_on = &mut self.notes_on;

    // Time Was Rewinded
    if self.last_time > time {
      // It is hard to track notes while rewinding, so we just turn all of them off
      for (id, _) in notes_on.iter() {
        let n = self.notes.get(id.to_owned());
        if let Some(n) = n {
          midi_out.send(&[0x80, n.note, n.vel]);
        };
      }
      notes_on.clear();
    }

    if let Some(note_renderer) = &self.note_renderer {
      note_renderer.draw(target, &public_state.viewport, time);
    }

    let mut active_notes: [bool; 88] = [false; 88];

    for n in self.notes.iter() {
      // Will Play In future Or Was Played Max 0.5s Ago
      if n.start <= time && n.start + n.duration + 0.5 > time {
        if n.start + n.duration >= time {
          active_notes[(n.note - 21) as usize] = true;

          if let std::collections::hash_map::Entry::Vacant(_e) = notes_on.entry(n.id) {
            notes_on.insert(n.id, true);
            midi_out.send(&[0x90, n.note, n.vel]);
          }
        } else {
          if let std::collections::hash_map::Entry::Occupied(_e) = notes_on.entry(n.id) {
            notes_on.remove(&n.id);
            midi_out.send(&[0x80, n.note, n.vel]);
          }
        }
      }
    }

    if self.notes.is_empty() {
      let menu = Box::new(crate::game_states::MenuState::new(self.display));
      return Some(menu);
    }

    self
      .keyboard
      .draw(target, &public_state.viewport, active_notes);

    self.last_time = time;
    None
  }

  fn prepare_drop(&mut self, public_state: &mut crate::render::PublicState) {
    let midi_out = &mut public_state.midi_device;
    let notes_on = &mut self.notes_on;

    // Turn off all notes before drop
    for (id, _) in notes_on.iter() {
      let n = self.notes.get(id.to_owned());
      if let Some(n) = n {
        midi_out.send(&[0x80, n.note, n.vel]);
      };
    }
  }
}
