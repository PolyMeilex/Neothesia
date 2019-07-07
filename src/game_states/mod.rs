#[path = "./playing_state/playing_state.rs"]
mod playing_state;

pub use playing_state::PlayingState;

pub enum StateUpdateMessage {
  PlayingState(Vec<crate::lib_midi::track::MidiNote>),
}

pub trait GameState<'a> {
  fn update(&mut self,msg:StateUpdateMessage);
  fn draw(&self, target: &mut glium::Frame, game_renderer: &crate::render::GameRenderer);
}