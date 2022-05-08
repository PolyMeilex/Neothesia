#[cfg(feature = "app")]
pub mod menu_scene;

pub mod playing_scene;

#[cfg(feature = "app")]
pub mod scene_transition;

use crate::target::Target;
use std::time::Duration;
use winit::event::WindowEvent;

pub trait Scene {
    fn scene_type(&self) -> SceneType;

    fn start(&mut self) {}
    fn done(self: Box<Self>, _target: &mut Target) {}

    fn resize(&mut self, _target: &mut Target) {}
    fn update(&mut self, target: &mut Target, delta: Duration);
    fn render(&mut self, target: &mut Target, view: &wgpu::TextureView);
    fn window_event(&mut self, _target: &mut Target, _event: &WindowEvent) {}
    fn main_events_cleared(&mut self, _target: &mut Target) {}
}

#[derive(Debug)]
pub enum SceneType {
    MainMenu,
    Playing,
    Transition,
}
