use crate::ui::Ui;
use crate::wgpu_jumpstart::Gpu;
use crate::MainState;

use winit::event::VirtualKeyCode;

pub trait Scene {
    fn state_type(&self) -> SceneType;
    fn resize(&mut self, _state: &mut MainState, _gpu: &mut Gpu) {}
    fn update(&mut self, state: &mut MainState, gpu: &mut Gpu, ui: &mut Ui) -> SceneEvent;
    fn render(&mut self, state: &mut MainState, gpu: &mut Gpu, frame: &wgpu::SwapChainOutput);
    fn key_released(&mut self, _state: &mut MainState, key: VirtualKeyCode) {}
}

pub enum SceneType {
    MainMenu,
    Playing,
}

pub enum SceneEvent {
    MainMenu(super::menu_scene::Event),
    None,
}
