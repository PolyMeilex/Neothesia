mod midi_backend;
mod synth_backend;

use midi_backend::{MidiBackend, MidiPortInfo};
use synth_backend::SynthBackend;

use std::{
    fmt::{self, Display, Formatter},
    path::Path,
};

#[derive(Debug, Clone)]
pub enum OutputDescriptor {
    Synth,
    MidiOut(MidiPortInfo),
    DummyOutput,
}

impl Display for OutputDescriptor {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            OutputDescriptor::Synth => write!(f, "Buildin Synth"),
            OutputDescriptor::MidiOut(info) => write!(f, "{}", info),
            OutputDescriptor::DummyOutput => write!(f, "No Output"),
        }
    }
}

pub trait OutputConnection {
    fn note_on(&mut self, _ch: u8, _key: u8, _vel: u8) {}
    fn note_off(&mut self, _ch: u8, _key: u8) {}
}

struct DummyOutput {}
impl OutputConnection for DummyOutput {}

pub struct OutputManager {
    synth_backend: Option<SynthBackend>,
    midi_backend: Option<MidiBackend>,

    output_connection: Box<dyn OutputConnection>,
}

impl OutputManager {
    pub fn new() -> Self {
        let synth_backend = if Path::new("./font.sf2").exists() {
            Some(SynthBackend::new())
        } else {
            log::info!("./font.sf2 not found");
            None
        };

        let midi_backend = match MidiBackend::new() {
            Ok(midi_device_manager) => Some(midi_device_manager),
            Err(e) => {
                log::error!("{:?}", e);
                None
            }
        };

        Self {
            synth_backend,
            midi_backend,

            output_connection: Box::new(DummyOutput {}),
        }
    }

    pub fn get_outputs(&self) -> Vec<OutputDescriptor> {
        let mut outs = Vec::new();

        if let Some(synth) = &self.synth_backend {
            outs.append(&mut synth.get_outputs());
        }
        if let Some(midi) = &self.midi_backend {
            outs.append(&mut midi.get_outputs());
        }

        outs.push(OutputDescriptor::DummyOutput);

        outs
    }

    pub fn connect(&mut self, desc: OutputDescriptor) {
        match desc {
            OutputDescriptor::Synth => {
                if let Some(ref mut synth) = self.synth_backend {
                    self.output_connection = Box::new(synth.new_output_connection());
                }
            }
            OutputDescriptor::MidiOut(info) => {
                if let Some(conn) = MidiBackend::new_output_connection(info) {
                    self.output_connection = Box::new(conn);
                }
            }
            OutputDescriptor::DummyOutput => {
                self.output_connection = Box::new(DummyOutput {});
            }
        }
    }

    pub fn note_on(&mut self, ch: u8, key: u8, vel: u8) {
        self.output_connection.note_on(ch, key, vel);
    }

    pub fn note_off(&mut self, ch: u8, key: u8) {
        self.output_connection.note_off(ch, key);
    }
}
