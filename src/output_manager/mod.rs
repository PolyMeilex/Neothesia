mod midi_backend;
mod synth_backend;

use midi_backend::{MidiBackend, MidiPortInfo};
use synth_backend::SynthBackend;

use std::{
    fmt::{self, Display, Formatter},
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, PartialEq)]
pub enum OutputDescriptor {
    Synth(Option<PathBuf>),
    MidiOut(MidiPortInfo),
    DummyOutput,
}

impl Display for OutputDescriptor {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            OutputDescriptor::Synth(_) => write!(f, "Buildin Synth"),
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

    output_connection: (OutputDescriptor, Box<dyn OutputConnection>),

    pub selected_output_id: Option<usize>,
}

impl OutputManager {
    pub fn new() -> Self {
        let synth_backend = match SynthBackend::new() {
            Ok(synth_backend) => Some(synth_backend),
            Err(err) => {
                log::error!("{:?}", err);
                None
            }
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

            output_connection: (OutputDescriptor::DummyOutput, Box::new(DummyOutput {})),
            selected_output_id: None,
        }
    }

    pub fn get_outputs(&self) -> Vec<OutputDescriptor> {
        let mut outs = Vec::new();

        outs.push(OutputDescriptor::DummyOutput);

        if let Some(synth) = &self.synth_backend {
            outs.append(&mut synth.get_outputs());
        }
        if let Some(midi) = &self.midi_backend {
            outs.append(&mut midi.get_outputs());
        }

        outs
    }

    pub fn connect(&mut self, desc: OutputDescriptor) {
        if desc != self.output_connection.0 {
            match desc {
                OutputDescriptor::Synth(ref font) => {
                    if let Some(ref mut synth) = self.synth_backend {
                        if let Some(font) = font.clone() {
                            self.output_connection =
                                (desc, Box::new(synth.new_output_connection(font)));
                        } else {
                            if Path::new("./default.sf2").exists() {
                                self.output_connection = (
                                    desc,
                                    Box::new(synth.new_output_connection("./default.sf2".into())),
                                );
                            }
                        }
                    }
                }
                OutputDescriptor::MidiOut(ref info) => {
                    if let Some(conn) = MidiBackend::new_output_connection(info) {
                        self.output_connection = (desc, Box::new(conn));
                    }
                }
                OutputDescriptor::DummyOutput => {
                    self.output_connection = (desc, Box::new(DummyOutput {}));
                }
            }
        }
    }

    pub fn note_on(&mut self, ch: u8, key: u8, vel: u8) {
        self.output_connection.1.note_on(ch, key, vel);
    }

    pub fn note_off(&mut self, ch: u8, key: u8) {
        self.output_connection.1.note_off(ch, key);
    }
}
