use crate::ui::Ui;
use crate::wgpu_jumpstart::Gpu;
use crate::MainState;

use winit::event::{ElementState, MouseButton, VirtualKeyCode};

pub trait Scene {
    fn scene_type(&self) -> SceneType;
    fn start(&mut self) {}
    fn resize(&mut self, _state: &mut MainState, _gpu: &mut Gpu) {}
    fn update(&mut self, state: &mut MainState, gpu: &mut Gpu, ui: &mut Ui) -> SceneEvent;
    fn render(&mut self, state: &mut MainState, gpu: &mut Gpu, view: &wgpu::TextureView);
    fn input_event(&mut self, _state: &mut MainState, _event: InputEvent) -> SceneEvent {
        SceneEvent::None
    }
}

#[derive(Debug)]
pub enum InputEvent<'a> {
    KeyReleased(VirtualKeyCode),
    MouseInput(&'a ElementState, &'a MouseButton),
    CursorMoved(f32, f32),
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
    None,
}
