use crate::ui::Ui;
use crate::wgpu_jumpstart::Gpu;
use crate::MainState;

use winit::event::{ElementState, MouseButton, VirtualKeyCode};

pub trait Scene {
    fn scene_type(&self) -> SceneType;
    fn start(&mut self) {}
    fn resize(&mut self, _state: &mut MainState, _gpu: &mut Gpu) {}
    fn update(&mut self, state: &mut MainState, gpu: &mut Gpu, ui: &mut Ui) -> SceneEvent;
    fn render(&mut self, state: &mut MainState, gpu: &mut Gpu, frame: &wgpu::SwapChainOutput);
    fn mouse_input(&mut self, _state: &ElementState, _button: &MouseButton) {}
    fn input_event(&mut self, _state: &mut MainState, _event: InputEvent) {}
}

pub enum InputEvent {
    KeyReleased(VirtualKeyCode),
}

pub enum SceneType {
    MainMenu,
    Playing,
    Transition,
}

pub enum SceneEvent {
    MainMenu(super::menu_scene::Event),
    None,
}
