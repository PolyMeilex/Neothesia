use std::{error::Error, fmt};

/// An error that can occur during initialization (i.e., while
/// creating a `MidiInput` or `MidiOutput` object).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InitError;

impl Error for InitError {}

impl fmt::Display for InitError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        "MIDI support could not be initialized".fmt(f)
    }
}

impl From<midir::InitError> for InitError {
    fn from(_: midir::InitError) -> Self {
        Self
    }
}

pub struct MidiOutputManager {
    output: midir::MidiOutput,
}

impl MidiOutputManager {
    pub fn new() -> Result<Self, InitError> {
        let output = midir::MidiOutput::new("MidiIo-out-manager")?;

        Ok(Self { output })
    }

    pub fn outputs(&self) -> Vec<MidiOutputPort> {
        self.output
            .ports()
            .iter()
            .filter_map(|p| self.output.port_name(p).ok())
            .map(MidiOutputPort)
            .collect()
    }

    pub fn connect_output(port: MidiOutputPort) -> Option<MidiOutputConnection> {
        let output = midir::MidiOutput::new("MidiIo-out").unwrap();

        let port = output.ports().into_iter().find(|info| {
            output
                .port_name(info)
                .ok()
                .map(|name| name == port.0)
                .unwrap_or(false)
        });

        port.and_then(move |port| output.connect(&port, "MidiIo-in-conn").ok())
            .map(MidiOutputConnection)
    }
}

pub struct MidiInputManager;

impl MidiInputManager {
    pub fn new() -> Result<Self, InitError> {
        let _ = midir::MidiInput::new("MidiIo-in-manager")?;
        Ok(Self)
    }

    pub fn inputs(&self) -> Vec<MidiInputPort> {
        let input = match midir::MidiInput::new("MidiIo-in-enum") {
            Ok(i) => i,
            Err(_) => return Vec::new(),
        };

        input
            .ports()
            .iter()
            .enumerate()
            .filter_map(|(index, p)| {
                input
                    .port_name(p)
                    .ok()
                    .map(|name| MidiInputPort { name, index })
            })
            .collect()
    }

    pub fn connect_input<F>(port: MidiInputPort, mut callback: F) -> Option<MidiInputConnection>
    where
        F: FnMut(&[u8]) + Send + 'static,
    {
        let input = midir::MidiInput::new("MidiIo-in").unwrap();
        let ports = input.ports();

        let mut chosen = None;

        // Prefer index if valid
        if port.index != usize::MAX {
            if let Some(p) = ports.get(port.index) {
                if input.port_name(p).ok().as_deref() == Some(port.name.as_str()) {
                    chosen = Some(p);
                }
            }
        }

        // Fallback by name
        let chosen = chosen.or_else(|| {
            ports.iter().find(|p| input.port_name(p).ok().as_deref() == Some(port.name.as_str()))
        });

        chosen
            .and_then(|p| {
                input.connect(
                    p,
                    "MidiIo-in-conn",
                    move |_, data, _| callback(data),
                    (),
                ).ok()
            })
            .map(MidiInputConnection)
    }


    pub fn has_input_port(&self, port: &MidiInputPort) -> bool {
        let input = match midir::MidiInput::new("MidiIo-in-has") {
            Ok(i) => i,
            Err(_) => return false,
        };

        let ports = input.ports();

        // index+name fast path
        if let Some(p) = ports.get(port.index) {
            if input.port_name(p).ok().as_deref() == Some(port.name.as_str()) {
                return true;
            }
        }

        // fallback by name
        ports.iter().any(|p| input.port_name(p).ok().as_deref() == Some(port.name.as_str()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MidiOutputPort(String);

impl std::fmt::Display for MidiOutputPort {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MidiInputPort {
    pub(crate) name: String,
    pub(crate) index: usize,
}

impl std::fmt::Display for MidiInputPort {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[allow(unused)]
pub struct MidiInputConnection(midir::MidiInputConnection<()>);
pub struct MidiOutputConnection(midir::MidiOutputConnection);

impl MidiOutputConnection {
    /// Send a message to the port that this output connection is connected to.
    /// The message must be a valid MIDI message (see https://www.midi.org/specifications-old/item/table-1-summary-of-midi-message).
    pub fn send(&mut self, message: &[u8]) -> Result<(), SendError> {
        self.0.send(message)?;
        Ok(())
    }
}

/// An error that can occur when sending MIDI messages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SendError {
    InvalidData(&'static str),
    Other(&'static str),
}

impl Error for SendError {}

impl fmt::Display for SendError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            SendError::InvalidData(msg) | SendError::Other(msg) => msg.fmt(f),
        }
    }
}

impl From<midir::SendError> for SendError {
    fn from(err: midir::SendError) -> Self {
        match err {
            midir::SendError::InvalidData(e) => Self::InvalidData(e),
            midir::SendError::Other(e) => Self::Other(e),
        }
    }
}

impl MidiInputPort {
    pub fn from_name(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            index: usize::MAX, // sentinel meaning “unknown index”
        }
    }
}
