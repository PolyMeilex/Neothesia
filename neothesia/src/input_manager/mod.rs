use midi_file::midly::{self, MidiMessage, live::LiveEvent};
use winit::event_loop::EventLoopProxy;

use crate::NeothesiaEvent;

pub struct InputManager {
    input: midi_io::MidiInputManager,
    tx: EventLoopProxy<NeothesiaEvent>,
    current_connection: Option<midi_io::MidiInputConnection>,
    current_port_name: Option<String>,
}

impl InputManager {
    pub fn new(tx: EventLoopProxy<NeothesiaEvent>) -> Self {
        let input = midi_io::MidiInputManager::new().unwrap();
        Self {
            input,
            tx,
            current_connection: None,
            current_port_name: None,
        }
    }

    pub fn inputs(&self) -> Vec<midi_io::MidiInputPort> {
        self.input.inputs()
    }

    pub fn connect_input(&mut self, port: midi_io::MidiInputPort) {
        let port_name = port.to_string();

        // Skip reconnect if already connected to same port
        if self.current_connection.is_some()
            && self.current_port_name.as_deref() == Some(port_name.as_str())
        {
            log::info!("connect_input: already connected to {port_name}, skipping");
            return;
        }

        // Explicitly drop previous connection
        self.current_connection.take();
        self.current_port_name = Some(port_name.clone());

        log::info!("connect_input: connecting to {port_name}");

        let tx = self.tx.clone();
        self.current_connection = midi_io::MidiInputManager::connect_input(port, move |message| {
            let Ok(event) = LiveEvent::parse(message) else { return; };

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
