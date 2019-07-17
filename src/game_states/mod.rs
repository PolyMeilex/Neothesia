#[path = "./playing_state/playing_state.rs"]
mod playing_state;
pub use playing_state::PlayingState;

#[path = "./menu_state/menu_state.rs"]
mod menu_state;
pub use menu_state::MenuState;

use std::ffi::c_void;

#[derive(Clone,Copy)]
pub enum GameStateType{
  menu_state,
  playing_state
}

pub trait GameState<'a> {
  fn get_type(&self) -> GameStateType;
  fn draw(
    &mut self,
    target: &mut glium::Frame,
    public_state: &mut crate::render::PublicState,
  ) -> Option<Box<dyn GameState<'a> + 'a>>;
  fn get_void_pointer(&mut self) -> *mut c_void {
    self as *mut _ as *mut std::ffi::c_void
  }
}