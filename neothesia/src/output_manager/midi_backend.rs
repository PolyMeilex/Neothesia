use std::collections::HashSet;

use crate::output_manager::{OutputConnection, OutputDescriptor};

use midi_file::midly::{
    self,
    live::LiveEvent,
    num::{u4, u7},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct ActiveNote {
    key: u7,
    channel: u4,
}

pub struct MidiOutputConnection {
    conn: midi_io::MidiOutputConnection,
    active_notes: HashSet<ActiveNote>,
    buf: Vec<u8>,
}

impl From<midi_io::MidiOutputConnection> for MidiOutputConnection {
    fn from(conn: midi_io::MidiOutputConnection) -> Self {
        Self {
            conn,
            active_notes: Default::default(),
            buf: Vec::with_capacity(8),
        }
    }
}

pub struct MidiBackend {
    manager: midi_io::MidiOutputManager,
}

impl MidiBackend {
    pub fn new() -> Result<Self, midi_io::InitError> {
        Ok(Self {
            manager: midi_io::MidiOutputManager::new()?,
        })
    }

    pub fn get_outputs(&self) -> Vec<OutputDescriptor> {
        let mut outs = Vec::new();
        for (id, port) in self.manager.outputs().into_iter().enumerate() {
            outs.push(OutputDescriptor::MidiOut(MidiPortInfo { id, port }))
        }
        outs
    }

    pub fn new_output_connection(port: &MidiPortInfo) -> Option<MidiOutputConnection> {
        midi_io::MidiOutputManager::connect_output(port.port.clone())
            .map(MidiOutputConnection::from)
    }
}

impl OutputConnection for MidiOutputConnection {
    fn midi_event(&mut self, channel: u4, message: midly::MidiMessage) {
        match message {
            midly::MidiMessage::NoteOff { key, .. } => {
                self.active_notes.remove(&ActiveNote { key, channel });
            }
            midly::MidiMessage::NoteOn { key, .. } => {
                self.active_notes.insert(ActiveNote { key, channel });
            }
            _ => {}
        }

        self.buf.clear();
        let msg = midly::live::LiveEvent::Midi { channel, message };
        msg.write(&mut self.buf).unwrap();

        self.conn.send(&self.buf).ok();
    }

    fn stop_all(&mut self) {
        for note in std::mem::take(&mut self.active_notes).iter() {
            self.buf.clear();
            let msg = LiveEvent::Midi {
                channel: note.channel,
                message: midly::MidiMessage::NoteOff {
                    key: note.key,
                    vel: u7::new(0),
                },
            };
            msg.write(&mut self.buf).unwrap();

            self.conn.send(&self.buf).ok();
        }
    }
}

impl Drop for MidiOutputConnection {
    fn drop(&mut self) {
        self.stop_all();
    }
}

#[derive(Clone, Debug, Eq)]
pub struct MidiPortInfo {
    id: usize,
    port: midi_io::MidiOutputPort,
}

impl PartialEq for MidiPortInfo {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.port == other.port
    }
}

impl std::fmt::Display for MidiPortInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.port)
    }
}
