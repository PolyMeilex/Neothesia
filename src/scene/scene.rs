use crate::ui::Ui;
use crate::wgpu_jumpstart::Gpu;
use crate::MainState;

use winit::event::WindowEvent;

pub trait Scene {
    fn scene_type(&self) -> SceneType;
    fn start(&mut self) {}
    fn resize(&mut self, _state: &mut MainState, _gpu: &mut Gpu) {}
    fn update(&mut self, state: &mut MainState, gpu: &mut Gpu, ui: &mut Ui) -> SceneEvent;
    fn render(&mut self, state: &mut MainState, gpu: &mut Gpu, frame: &wgpu::SwapChainOutput);
    fn window_event(&mut self, _state: &mut MainState, _event: &WindowEvent) -> SceneEvent {
        SceneEvent::None
    }
    fn main_events_cleared(&mut self, _state: &mut MainState) -> SceneEvent {
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
