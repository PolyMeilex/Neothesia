pub mod menu_scene;
pub mod playing_scene;

use crate::target::Target;
use midi_file::midly::MidiMessage;
use std::time::Duration;
use wgpu_jumpstart::{TransformUniform, Uniform};
use winit::event::WindowEvent;

pub trait Scene {
    fn resize(&mut self, _target: &mut Target) {}
    fn update(&mut self, target: &mut Target, delta: Duration);
    fn render<'pass>(
        &'pass mut self,
        transform: &'pass Uniform<TransformUniform>,
        rpass: &mut wgpu::RenderPass<'pass>,
    );
    fn window_event(&mut self, _target: &mut Target, _event: &WindowEvent) {}
    fn midi_event(&mut self, _target: &mut Target, _channel: u8, _message: &MidiMessage) {}
}
