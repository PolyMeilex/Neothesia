use crate::output_manager::{OutputConnection, OutputDescriptor};

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
        for p in ports {
            let name = match self.midi_out.port_name(&p).ok() {
                Some(name) => name,
                None => String::from("Unknown"),
            };
            outs.push(OutputDescriptor::MidiOut(MidiPortInfo { port: p, name }))
        }
        outs
    }

    pub fn new_output_connection(port: MidiPortInfo) -> Option<MidiOutputConnection> {
        let midi_out = MidiOutput::new("midi_out_conn").ok();

        if let Some(midi_out) = midi_out {
            midi_out.connect(&port.port, "out").ok()
        } else {
            None
        }
    }
}

impl OutputConnection for MidiOutputConnection {
    fn note_on(&mut self, _ch: u8, key: u8, vel: u8) {
        self.send(&[0x90, key, vel]).ok();
    }
    fn note_off(&mut self, _ch: u8, key: u8) {
        self.send(&[0x80, key, 0]).ok();
    }
}

#[derive(Clone)]
pub struct MidiPortInfo {
    pub port: MidiOutputPort,
    pub name: String,
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
