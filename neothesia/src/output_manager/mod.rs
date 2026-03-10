mod midi_backend;
use midi_backend::{MidiBackend, MidiPortInfo};

#[cfg(feature = "synth")]
mod synth_backend;

#[cfg(feature = "synth")]
use synth_backend::SynthBackend;

use std::{
    fmt::{self, Display, Formatter},
    path::PathBuf,
};

use midi_file::midly::{MidiMessage, num::u4};

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum OutputDescriptor {
    #[cfg(feature = "synth")]
    Synth(Option<PathBuf>),
    MidiOut(MidiPortInfo),
    DummyOutput,
}

impl OutputDescriptor {
    pub fn is_dummy(&self) -> bool {
        matches!(self, Self::DummyOutput)
    }

    pub fn is_not_dummy(&self) -> bool {
        !self.is_dummy()
    }

    pub fn is_midi(&self) -> bool {
        matches!(self, OutputDescriptor::MidiOut(_))
    }

    pub fn is_synth(&self) -> bool {
        matches!(self, OutputDescriptor::Synth(_))
    }
}

impl Display for OutputDescriptor {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            #[cfg(feature = "synth")]
            OutputDescriptor::Synth(_) => write!(f, "Buildin Synth"),
            OutputDescriptor::MidiOut(info) => write!(f, "{info}"),
            OutputDescriptor::DummyOutput => write!(f, "No Output"),
        }
    }
}

#[derive(Clone)]
pub enum OutputConnection {
    Midi(midi_backend::MidiOutputConnection),
    #[cfg(feature = "synth")]
    Synth(synth_backend::SynthOutputConnection),
    DummyOutput,
}

impl OutputConnection {
    pub fn midi_event(&self, channel: u4, msg: MidiMessage) {
        match self {
            OutputConnection::Midi(b) => b.midi_event(channel, msg),
            #[cfg(feature = "synth")]
            OutputConnection::Synth(b) => b.midi_event(channel, msg),
            OutputConnection::DummyOutput => {}
        }
    }
    pub fn send_sysex(&self, message: &[u8]) {
        match self {
            OutputConnection::Midi(b) => b.send_sysex(message),
            _ => {}
        }
    }
    pub fn set_gain(&self, gain: f32) {
        match self {
            #[cfg(feature = "synth")]
            OutputConnection::Synth(b) => b.set_gain(gain),
            _ => {}
        }
    }
    pub fn stop_all(&self) {
        match self {
            OutputConnection::Midi(b) => b.stop_all(),
            #[cfg(feature = "synth")]
            OutputConnection::Synth(b) => b.stop_all(),
            OutputConnection::DummyOutput => {}
        }
    }
}

pub struct OutputManager {
    #[cfg(feature = "synth")]
    synth_backend: Option<SynthBackend>,
    midi_backend: Option<MidiBackend>,

    output_connection: (OutputDescriptor, OutputConnection),

    /// Dedicated MIDI output connection used exclusively for LUMI SysEx and LED control.
    /// Separate from the audio output so the user can keep the synth for sound
    /// while still driving the LUMI LEDs.
    lumi_connection: Option<midi_backend::MidiOutputConnection>,
}

impl Default for OutputManager {
    fn default() -> Self {
        Self::new()
    }
}

impl OutputManager {
    pub fn new() -> Self {
        #[cfg(feature = "synth")]
        let synth_backend = match SynthBackend::new() {
            Ok(synth_backend) => Some(synth_backend),
            Err(err) => {
                log::error!("{err:?}");
                None
            }
        };

        let midi_backend = match MidiBackend::new() {
            Ok(midi_device_manager) => Some(midi_device_manager),
            Err(e) => {
                log::error!("{e}");
                None
            }
        };

        Self {
            #[cfg(feature = "synth")]
            synth_backend,
            midi_backend,

            output_connection: (OutputDescriptor::DummyOutput, OutputConnection::DummyOutput),
            lumi_connection: None,
        }
    }

    pub fn outputs(&self) -> Vec<OutputDescriptor> {
        let mut outs = Vec::new();

        #[cfg(feature = "synth")]
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
        if desc != self.output_connection.0 {
            match desc {
                #[cfg(feature = "synth")]
                OutputDescriptor::Synth(ref font) => {
                    if let Some(ref mut synth) = self.synth_backend {
                        if let Some(font) = font.clone() {
                            self.output_connection = (
                                desc,
                                OutputConnection::Synth(synth.new_output_connection(&font)),
                            );
                        } else if let Some(path) = crate::utils::resources::default_sf2()
                            && path.exists()
                        {
                            self.output_connection = (
                                desc,
                                OutputConnection::Synth(synth.new_output_connection(&path)),
                            );
                        }
                    }
                }
                OutputDescriptor::MidiOut(ref info) => {
                    if let Some(conn) = MidiBackend::new_output_connection(info) {
                        self.output_connection = (desc, OutputConnection::Midi(conn));
                    }
                }
                OutputDescriptor::DummyOutput => {
                    self.output_connection = (desc, OutputConnection::DummyOutput);
                }
            }
        }
    }

    pub fn connection(&self) -> &OutputConnection {
        &self.output_connection.1
    }

    /// Open a dedicated MIDI output connection to the LUMI device.
    /// Matches by port name (the LUMI appears with identical names on input and output).
    /// This is independent of the main audio output connection.
    pub fn connect_lumi_by_port_name(&mut self, port_name: &str) {
        let Some(backend) = &self.midi_backend else { return; };
        let outputs = backend.get_outputs();

        // The LUMI port name on input and output might differ slightly; try exact match first,
        // then substring match (e.g. "LUMI Keys" appears in both directions).
        let found = outputs.iter().find(|o| o.to_string() == port_name)
            .or_else(|| {
                let lower = port_name.to_lowercase();
                outputs.iter().find(|o| {
                    let n = o.to_string().to_lowercase();
                    n.contains("lumi") || lower.contains(&n) || n.contains(&lower)
                })
            });

        if let Some(OutputDescriptor::MidiOut(info)) = found {
            self.lumi_connection = MidiBackend::new_output_connection(info);
            log::info!("LUMI SysEx output connected: {}", info);
        } else {
            log::warn!("LUMI SysEx output not found for input port: {}", port_name);
        }
    }

    pub fn disconnect_lumi(&mut self) {
        self.lumi_connection = None;
    }

    /// Returns the connection to use for LUMI SysEx.
    /// Prefers the dedicated LUMI connection; falls back to the main output if it is a MIDI port.
    pub fn lumi_connection(&self) -> OutputConnection {
        if let Some(ref c) = self.lumi_connection {
            return OutputConnection::Midi(c.clone());
        }
        // Fallback: main output if it happens to be MIDI
        self.output_connection.1.clone()
    }
}
