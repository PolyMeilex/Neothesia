use crate::utils;
use glium::Surface;

use crate::game_states;
use crate::game_states::GameState;

use crate::midi_device::MidiDevice;

use crate::render::ui::UiRenderer;

pub struct PublicState<'a> {
  pub viewport: glium::Rect,
  pub time: f64,
  pub ui_renderer: UiRenderer<'a>,
  pub m_pos: utils::Vec2,
  pub m_pressed: bool,
  pub m_was_pressed: bool,
  pub midi_device: MidiDevice,
}

pub struct GameRenderer<'a> {
  pub public_state: PublicState<'a>,

  display: &'a glium::Display,
  game_state: Box<dyn GameState<'a> + 'a>,

  pub fps: u64,
  ar: f32,
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
        midi_device: MidiDevice::new(),
      },

      display,
      game_state: Box::new(game_states::MenuState::new(display)),

      fps: 0,
      ar: 16.0 / 9.0,

      update_size: true,
    }
  }
  pub fn get_state_type(&self) -> game_states::GameStateType {
    self.game_state.get_type()
  }
  pub fn set_state(&mut self, new_state: Box<dyn GameState<'a> + 'a>) {
    self.game_state.prepare_drop(&mut self.public_state);
    self.game_state = new_state;
  }
  pub fn state_go_back(&mut self) -> bool {
    let state_type = self.get_state_type();
    let mut closed = false;

    match state_type {
      game_states::GameStateType::PlayingState => {
        self.set_state(Box::new(game_states::MenuState::new(self.display)));
      }
      game_states::GameStateType::MenuState => {
        closed = true;
      }
    };

    closed
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
      } else {
        self.public_state.viewport.bottom = 0;
      }

      // No need to update it on every frame, when window has same size
      self.update_size = false;
    }

    target.clear_color_srgb(0.1, 0.1, 0.1, 1.0);
    target.clear_depth(1.0);

    let new_state = self.game_state.draw(&mut target, &mut self.public_state);

    self.public_state.ui_renderer.text_writer.add(
      &self.fps.to_string(),
      0.0,
      self.public_state.viewport.bottom as f32,
      false,
      None,
    );

    self.public_state.ui_renderer.draw(&mut target);

    target.finish().unwrap();

    if let Some(state_box) = new_state {
       self.set_state(state_box);
    }

    // m_was_pressed is true when mouse was clicked this frame
    self.public_state.m_was_pressed = false;
  }
}
