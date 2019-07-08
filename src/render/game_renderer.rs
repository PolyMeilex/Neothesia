use crate::utils;
use glium::Surface;

use crate::game_states;
use crate::game_states::GameState;

// use crate::render::KeyboardRenderer;
// use crate::render::NoteRenderer;
use crate::render::ui::UiRenderer;

pub struct PublicState<'a> {
  pub viewport: glium::Rect,
  pub time: f64,
  pub ui_renderer: UiRenderer<'a>,
  pub m_pos: utils::Vec2,
  pub m_pressed: bool,
  pub m_was_pressed: bool,
}

pub struct GameRenderer<'a> {
  pub public_state: PublicState<'a>,

  display: &'a glium::Display,
  game_state: Box<GameState<'a> + 'a>,

  // note_renderer: Option<NoteRenderer<'a>>,
  // keyboard_renderer: KeyboardRenderer<'a>,


  // notes: Vec<crate::lib_midi::track::MidiNote>,

  pub fps: u64,
  // pub time: f64,
  ar: f32,

  // pub viewport: glium::Rect,
  pub update_size: bool,
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
      public_state: PublicState {
        viewport,
        time: 0.0,
        ui_renderer: UiRenderer::new(display),
        m_pos: utils::Vec2 { x: 0.0, y: 0.0 },
        m_pressed: false,
        m_was_pressed: false,
      },

      display,
      // viewport,

      game_state: Box::new(game_states::MenuState::new(display)),

      // note_renderer: None,
      // keyboard_renderer: KeyboardRenderer::new(display),


      // notes: Vec::new(),

      fps: 0,
      // time: 0.0,
      ar: 16.0 / 9.0,

      update_size: true,

    }
  }
  pub fn set_state(&mut self, state: Box<GameState<'a> + 'a>) {
    self.game_state = state;
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

    self.public_state.time = time;

    let mut target = self.display.draw();

    if self.update_size {
      let (width, height) = target.get_dimensions();

      self.public_state.viewport.width = width;
      self.public_state.viewport.height = (width as f32 / self.ar) as u32;

      if height >= self.public_state.viewport.height {
        self.public_state.viewport.bottom = (height - self.public_state.viewport.height) / 2;
      }

      // No need to update it on every frame, when window has same size
      self.update_size = false;
    }

    target.clear_color_srgb(0.1, 0.1, 0.1, 1.0);


    let new_state = self.game_state.draw(&mut target, &self.public_state);

    self.public_state.ui_renderer.text_writer.add(
      &self.fps.to_string(),
      0.0,
      self.public_state.viewport.bottom as f32,
    );

    self.public_state.ui_renderer.draw(&mut target);

    target.finish().unwrap();

    match new_state {
      Some(state_box) => {
        self.game_state = state_box;
      }
      None => {}
    }

    // m_was_pressed is true when mouse was clicked this frame
    self.public_state.m_was_pressed = false;
  }
}

