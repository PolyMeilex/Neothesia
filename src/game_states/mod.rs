#[path = "./playing_state/playing_state.rs"]
mod playing_state;
pub use playing_state::PlayingState;

#[path = "./menu_state/menu_state.rs"]
mod menu_state;
pub use menu_state::MenuState;

pub enum StateUpdateMessage {
  PlayingState(Vec<crate::lib_midi::track::MidiNote>),
}

pub trait GameState<'a> {
  fn update(&mut self,msg:StateUpdateMessage);
  fn draw(&mut self, target: &mut glium::Frame, public_state: &crate::render::PublicState) -> Option<Box<GameState<'a> + 'a>>;
}