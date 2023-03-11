#![allow(clippy::collapsible_match, clippy::single_match)]

pub use wgpu_jumpstart::{Gpu, TransformUniform, Uniform};

pub mod ui;

pub mod scene;

pub mod utils;

pub mod output_manager;
pub use output_manager::OutputManager;

pub mod input_manager;

pub mod config;

pub mod target;

pub mod keyboard_renderer;

pub mod midi_event;
use midi_event::MidiEvent;

use futures::Future;

#[derive(Debug, Clone)]
pub struct EventLoopProxy {
    #[cfg(feature = "app")]
    proxy: winit::event_loop::EventLoopProxy<NeothesiaEvent>,
}

impl EventLoopProxy {
    #[cfg(feature = "record")]
    pub fn new_mock() -> Self {
        Self {}
    }

    #[cfg(feature = "app")]
    pub fn new_winit(proxy: winit::event_loop::EventLoopProxy<NeothesiaEvent>) -> Self {
        Self { proxy }
    }

    #[track_caller]
    pub fn send_event(&self, event: NeothesiaEvent) {
        #[cfg(feature = "app")]
        self.proxy.send_event(event).unwrap();
        #[cfg(feature = "record")]
        let _ = event;
    }
}

#[derive(Debug)]
pub enum NeothesiaEvent {
    #[cfg(feature = "app")]
    MainMenu(crate::scene::menu_scene::Event),
    MidiInput(MidiEvent),
    GoBack,
}

pub fn block_on<F>(f: F) -> <F as Future>::Output
where
    F: Future,
{
    futures::executor::block_on(f)
}
