pub mod menu_scene;
pub mod playing_scene;

use crate::target::Target;
use midi_file::midly::MidiMessage;
use std::time::Duration;
use winit::event::WindowEvent;

pub trait Scene {
    fn resize(&mut self, _target: &mut Target) {}
    fn update(&mut self, target: &mut Target, delta: Duration);
    fn render(&mut self, target: &mut Target, view: &wgpu::TextureView);
    fn window_event(&mut self, _target: &mut Target, _event: &WindowEvent) {}
    fn midi_event(&mut self, _target: &mut Target, _channel: u8, _message: &MidiMessage) {}
    fn main_events_cleared(&mut self, _target: &mut Target) {}
}
