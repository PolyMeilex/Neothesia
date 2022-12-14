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

pub mod midi_event;
use midi_event::MidiEvent;

use futures::Future;

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
