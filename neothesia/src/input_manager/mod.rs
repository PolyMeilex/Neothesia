use midi_file::midly::{self, live::LiveEvent};
use winit::event_loop::EventLoopProxy;

use crate::{midi_event::MidiEvent, NeothesiaEvent};

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
        self.current_connection = midi_io::MidiInputManager::connect_input(port, move |message| {
            let event = LiveEvent::parse(message).unwrap();

            if let LiveEvent::Midi { channel, message } = event {
                match message {
                    // Some keyboards send NoteOn event with vel 0 instead of NoteOff
                    midly::MidiMessage::NoteOn { key, vel } if vel == 0 => {
                        tx.send_event(NeothesiaEvent::MidiInput(MidiEvent::NoteOff {
                            channel: channel.as_int(),
                            key: key.as_int(),
                        }))
                        .ok();
                    }
                    midly::MidiMessage::NoteOn { key, vel } => {
                        tx.send_event(NeothesiaEvent::MidiInput(MidiEvent::NoteOn {
                            channel: channel.as_int(),
                            track_id: 0,
                            key: key.as_int(),
                            vel: vel.as_int(),
                        }))
                        .ok();
                    }
                    midly::MidiMessage::NoteOff { key, .. } => {
                        tx.send_event(NeothesiaEvent::MidiInput(MidiEvent::NoteOff {
                            channel: channel.as_int(),
                            key: key.as_int(),
                        }))
                        .ok();
                    }
                    _ => {}
                }
            }
        });
    }
}
