use std::time::Duration;

use crate::{
    midi_event::MidiEvent,
    scene::{Scene, SceneType},
    target::Target,
};

use winit::event::WindowEvent;

pub struct SceneManager {
    scene: Box<dyn Scene>,
}

impl SceneManager {
    pub fn new<S: Scene + 'static>(scene: S) -> Self {
        Self {
            scene: Box::new(scene),
        }
    }

    pub fn transition_to<S: Scene + 'static>(&mut self, target: &mut Target, scene: S) {
        let old = std::mem::replace(&mut self.scene, Box::new(scene));
        old.done(target);

        self.scene.start();
    }
}

impl SceneManager {
    pub fn scene_type(&self) -> SceneType {
        self.scene.scene_type()
    }

    pub fn resize(&mut self, target: &mut Target) {
        self.scene.resize(target)
    }

    pub fn update(&mut self, target: &mut Target, delta: Duration) {
        self.scene.update(target, delta)
    }

    pub fn render(&mut self, target: &mut Target, view: &wgpu::TextureView) {
        self.scene.render(target, view)
    }

    pub fn window_event(&mut self, target: &mut Target, event: &WindowEvent) {
        self.scene.window_event(target, event)
    }

    pub fn midi_event(&mut self, target: &mut Target, event: &MidiEvent) {
        self.scene.midi_event(target, event)
    }

    pub fn main_events_cleared(&mut self, target: &mut Target) {
        self.scene.main_events_cleared(target)
    }
}
