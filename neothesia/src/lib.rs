#![allow(clippy::collapsible_match, clippy::single_match)]

use futures::Future;
use midi_file::midly::MidiMessage;
pub use wgpu_jumpstart::{Gpu, TransformUniform, Uniform};

use neothesia_core::{config, render};
pub mod utils;

pub mod iced_utils;
pub mod input_manager;
pub mod output_manager;
pub mod scene;
pub mod target;

#[derive(Debug)]
pub enum NeothesiaEvent {
    MainMenu(crate::scene::menu_scene::Event),
    MidiInput {
        /// The MIDI channel that this message is associated with.
        channel: u8,
        /// The MIDI message type and associated data.
        message: MidiMessage,
    },
    GoBack,
}

pub fn block_on<F>(f: F) -> <F as Future>::Output
where
    F: Future,
{
    futures::executor::block_on(f)
}
