use crate::utils;
use glium::Surface;

use crate::game_states;
use crate::game_states::GameState;

use crate::render::ui::button::Button;

// use crate::render::KeyboardRenderer;
// use crate::render::NoteRenderer;
use crate::render::UiRenderer;

pub struct GameRenderer<'a> {
  display: &'a glium::Display,
  game_state: Box<GameState<'a> + 'a>,

  // note_renderer: Option<NoteRenderer<'a>>,
  // keyboard_renderer: KeyboardRenderer<'a>,
  ui_renderer: UiRenderer<'a>,

  // notes: Vec<crate::lib_midi::track::MidiNote>,

  pub fps: u64,
  pub time: f64,
  ar: f32,

  pub viewport: glium::Rect,
  pub update_size: bool,
  pub m_pos: utils::Vec2,
  pub m_pressed: bool,
}

impl<'a> GameRenderer<'a> {
  pub fn new(display: &'a glium::Display) -> Self {
    let viewport = glium::Rect {
      left: 0,
      bottom: 0,
      width: 1280,
      height: 720,
    };
    GameRenderer {
      display,
      viewport,

      game_state: Box::new(game_states::PlayingState::new(display)),

      // note_renderer: None,
      // keyboard_renderer: KeyboardRenderer::new(display),
      ui_renderer: UiRenderer::new(display),

      // notes: Vec::new(),

      fps: 0,
      time: 0.0,
      ar: 16.0 / 9.0,

      update_size: true,
      m_pos: utils::Vec2 { x: 0.0, y: 0.0 },
      m_pressed: false,
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

    self
      .game_state
      .update(crate::game_states::StateUpdateMessage::PlayingState(notes))
  }
  pub fn draw(&mut self, time: u128) {
    let time = time as f64 / 1000.0;

    self.time = time;

    let mut target = self.display.draw();

    if self.update_size {
      let (width, height) = target.get_dimensions();

      self.viewport.width = width;
      self.viewport.height = (width as f32 / self.ar) as u32;

      if height >= self.viewport.height {
        self.viewport.bottom = (height - self.viewport.height) / 2;
      }

      // No need to update it on every frame, when window has same size
      self.update_size = false;
    }

    target.clear_color_srgb(0.1, 0.1, 0.1, 1.0);

    self.game_state.draw(&mut target, self);

    // match &self.note_renderer {
    //   Some(note_renderer) => note_renderer.draw(&mut target, self, time as f32),
    //   None => {}
    // }


    // let mut active_notes: [bool; 88] = [false; 88];

    // // Causes a lot of lag when plaing Black Midi;
    // for n in self.notes.iter() {
    //   if n.start < time + 3.0 && n.start - time > -0.1 {
    //     if n.start - time < 0.0 {
    //       active_notes[(n.note - 21) as usize] = true;
    //     }
    //   }
    // }

    // self.keyboard_renderer.draw(&mut target, self, active_notes);

    self
      .ui_renderer
      .text_writer
      .add(&self.fps.to_string(), 0.0, self.viewport.bottom as f32);

    // let mut btn = Button {
    //   pos: utils::Vec2 { x: 0.0, y: 0.0 },
    //   size: utils::Vec2 { x: 0.3, y: 0.2 },
    //   hover: false,
    // };

    // btn.hover_check(&self.m_pos);


    // self
    //   .ui_renderer
    //   .buttons_renderer
    //   .draw(&mut target, self, btn);

    self.ui_renderer.draw(&mut target);

    target.finish().unwrap();
  }
}

