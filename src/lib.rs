#![allow(clippy::collapsible_match, clippy::single_match)]

use futures::Future;
pub use wgpu_jumpstart::{Gpu, TransformUniform, Uniform};

pub mod config;
pub mod ui;
pub mod utils;

pub mod keyboard_renderer;
pub mod waterfall_renderer;

#[cfg(feature = "app")]
pub mod scene;
#[cfg(feature = "app")]
pub mod output_manager;
#[cfg(feature = "app")]
pub mod input_manager;
#[cfg(feature = "app")]
pub mod target;
#[cfg(feature = "app")]
pub mod midi_event;

#[cfg(feature = "app")]
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
