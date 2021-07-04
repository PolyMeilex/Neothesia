use crate::output_manager::{OutputConnection, OutputDescriptor};

use midi::ToRawMessages;
use midir::{MidiOutput, MidiOutputConnection, MidiOutputPort};

pub struct MidiBackend {
    midi_out: MidiOutput,
}

impl MidiBackend {
    pub fn new() -> Result<Self, midir::InitError> {
        let midi_out = MidiOutput::new("midi_out")?;
        Ok(Self { midi_out })
    }

    pub fn get_outputs(&self) -> Vec<OutputDescriptor> {
        let mut outs = Vec::new();
        let ports = self.midi_out.ports();
        for (id, p) in ports.into_iter().enumerate() {
            let name = match self.midi_out.port_name(&p).ok() {
                Some(name) => name,
                None => String::from("Unknown"),
            };
            outs.push(OutputDescriptor::MidiOut(MidiPortInfo {
                id,
                port: p,
                name,
            }))
        }
        outs
    }

    pub fn new_output_connection(port: &MidiPortInfo) -> Option<MidiOutputConnection> {
        let midi_out = MidiOutput::new("midi_out_conn").ok();

        if let Some(midi_out) = midi_out {
            midi_out.connect(&port.port, "out").ok()
        } else {
            None
        }
    }
}

impl OutputConnection for MidiOutputConnection {
    fn midi_event(&mut self, msg: midi::Message) {
        if let Some(msg) = msg.to_raw_messages().first() {
            match *msg {
                midi::RawMessage::StatusDataData(a, b, c) => {
                    self.send(&[a, b, c]).ok();
                }
                _ => {}
            }
        }
    }
}

#[derive(Clone)]
pub struct MidiPortInfo {
    id: usize,
    port: MidiOutputPort,
    name: String,
}

impl PartialEq for MidiPortInfo {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.name == other.name
    }
}

impl std::fmt::Display for MidiPortInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl std::fmt::Debug for MidiPortInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}
