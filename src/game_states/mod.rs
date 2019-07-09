#[path = "./playing_state/playing_state.rs"]
mod playing_state;
pub use playing_state::PlayingState;

#[path = "./menu_state/menu_state.rs"]
mod menu_state;
pub use menu_state::MenuState;


pub trait GameState<'a> {
  fn draw(&mut self, target: &mut glium::Frame, public_state: &crate::render::PublicState) -> Option<Box<GameState<'a> + 'a>>;
}