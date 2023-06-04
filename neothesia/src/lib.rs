#![allow(clippy::collapsible_match, clippy::single_match)]

use futures::Future;
pub use wgpu_jumpstart::{Gpu, TransformUniform, Uniform};

use neothesia_core::{config, render};
pub mod utils;

pub mod iced_utils;
pub mod input_manager;
pub mod midi_event;
pub mod output_manager;
pub mod scene;
pub mod target;

#[derive(Debug)]
pub enum NeothesiaEvent {
    MainMenu(crate::scene::menu_scene::Event),
    MidiInput(midi_event::MidiEvent),
    GoBack,
}

pub fn block_on<F>(f: F) -> <F as Future>::Output
where
    F: Future,
{
    futures::executor::block_on(f)
}
