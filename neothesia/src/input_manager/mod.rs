use midi_file::midly::{self, MidiMessage, live::LiveEvent};
use winit::event_loop::EventLoopProxy;

use crate::NeothesiaEvent;

pub struct InputManager {
    input: midi_io::MidiInputManager,
    tx: EventLoopProxy<NeothesiaEvent>,
    current_connection: Option<midi_io::MidiInputConnection>,
}

impl InputManager {
    pub fn new(tx: EventLoopProxy<NeothesiaEvent>) -> Self {
        let input = midi_io::MidiInputManager::new().unwrap();
        Self {
            input,
            tx,
            current_connection: None,
        }
    }

    pub fn inputs(&self) -> Vec<midi_io::MidiInputPort> {
        self.input.inputs()
    }

    pub fn connect_input(&mut self, port: midi_io::MidiInputPort) {
        let tx = self.tx.clone();

        // Close the connection first, as Windows does not like it when we hold 2 connections
        self.current_connection = None;

        self.current_connection = midi_io::MidiInputManager::connect_input(port, move |message| {
            let event = LiveEvent::parse(message).unwrap();

            if let LiveEvent::Midi { channel, message } = event {
                match message {
                    // Some keyboards send NoteOn event with vel 0 instead of NoteOff
                    midly::MidiMessage::NoteOn { key, vel } if vel == 0 => {
                        tx.send_event(NeothesiaEvent::MidiInput {
                            channel: channel.as_int(),
                            message: MidiMessage::NoteOff { key, vel },
                        })
                        .ok();
                    }
                    message => {
                        tx.send_event(NeothesiaEvent::MidiInput {
                            channel: channel.as_int(),
                            message,
                        })
                        .ok();
                    }
                }
            }
        });
    }
}
