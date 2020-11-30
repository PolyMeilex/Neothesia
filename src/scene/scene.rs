use crate::MainState;
use crate::Target;

use winit::event::WindowEvent;

pub trait Scene {
    fn scene_type(&self) -> SceneType;

    fn start(&mut self) {}
    fn done(self: Box<Self>) -> MainState;

    fn resize(&mut self, _target: &mut Target) {}
    fn update(&mut self, target: &mut Target) -> SceneEvent;
    fn render(&mut self, target: &mut Target, frame: &wgpu::SwapChainFrame);
    fn window_event(&mut self, _target: &mut Target, _event: &WindowEvent) -> SceneEvent {
        SceneEvent::None
    }
    fn main_events_cleared(&mut self, _target: &mut Target) -> SceneEvent {
        SceneEvent::None
    }
}

#[derive(Debug)]
pub enum SceneType {
    MainMenu,
    Playing,
    Transition,
}

#[derive(Debug)]
pub enum SceneEvent {
    MainMenu(super::menu_scene::Event),
    GoBack,
    None,
}
