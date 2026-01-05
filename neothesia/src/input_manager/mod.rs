use midi_file::midly::{self, MidiMessage, live::LiveEvent};
use winit::event_loop::EventLoopProxy;

use crate::NeothesiaEvent;

pub struct InputManager {
    input: midi_io::MidiInputManager,
    tx: EventLoopProxy<NeothesiaEvent>,
    current_connection: Option<midi_io::MidiInputConnection>,
    current_port: Option<midi_io::MidiInputPort>, // store the actual port object
}

impl InputManager {
    pub fn new(tx: EventLoopProxy<NeothesiaEvent>) -> Self {
        let input = midi_io::MidiInputManager::new().unwrap();
        Self {
            input,
            tx,
            current_connection: None,
            current_port: None,
        }
    }

    pub fn inputs(&self) -> Vec<midi_io::MidiInputPort> {
        self.input.inputs()
    }

    pub fn connect_input(&mut self, port: midi_io::MidiInputPort) {
        // If we think we're connected, but the port no longer exists, drop connection.
        if self.current_connection.is_some() {
            if let Some(cur) = &self.current_port {
                if !self.input.has_input_port(cur) {
                    self.current_connection.take();
                    self.current_port = None;
                }
            }
        }

        // Skip reconnect if already connected to same port *and it still exists*
        if self.current_connection.is_some()
            && self.current_port.as_ref() == Some(&port)
            && self.input.has_input_port(&port)
        {
            return;
        }

        self.current_connection.take();
        self.current_port = Some(port.clone());

        let tx = self.tx.clone();
        self.current_connection = midi_io::MidiInputManager::connect_input(port, move |message| {
            let Ok(event) = LiveEvent::parse(message) else { return; };

            if let LiveEvent::Midi { channel, message } = event {
                match message {
                    midly::MidiMessage::NoteOn { key, vel } if vel == 0 => {
                        tx.send_event(NeothesiaEvent::MidiInput {
                            channel: channel.as_int(),
                            message: MidiMessage::NoteOff { key, vel },
                        }).ok();
                    }
                    message => {
                        tx.send_event(NeothesiaEvent::MidiInput {
                            channel: channel.as_int(),
                            message,
                        }).ok();
                    }
                }
            }
        });
    }

    pub fn force_reconnect(&mut self) {
        // Drop the connection and clear the "already connected" guard
        self.current_connection.take();
        self.current_port = None;
    }

    // Connect to the user's preferred input if it exists; otherwise connect to first available.
    // Returns the chosen port name (for storing back into config/UI if you want).
    pub fn connect_preferred_by_name(&mut self, preferred: Option<&str>) -> Option<String> {
        let inputs = self.inputs();

        // Try preferred
        if let Some(name) = preferred {
            if let Some(port) = inputs.iter().find(|p| p.to_string() == name).cloned() {
                self.connect_input(port);
                return Some(name.to_string());
            }
        }

        // Fallback to first
        if let Some(port) = inputs.first().cloned() {
            let chosen = port.to_string();
            self.connect_input(port);
            return Some(chosen);
        }

        None
    }
}
