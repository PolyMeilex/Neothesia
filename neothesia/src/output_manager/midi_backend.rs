use std::collections::HashSet;

use crate::output_manager::{OutputConnection, OutputDescriptor};

use midi_file::{
    midly::{
        self,
        live::LiveEvent,
        num::{u4, u7},
    },
    ActiveNote,
};

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
    fn midi_event(&mut self, msg: &midi_file::MidiEvent) {
        use midi_file::midly::MidiMessage;
        match &msg.message {
            MidiMessage::NoteOff { key, .. } => {
                self.active_notes.remove(&ActiveNote {
                    key: key.as_int(),
                    channel: msg.channel,
                });
            }
            MidiMessage::NoteOn { key, .. } => {
                self.active_notes.insert(ActiveNote {
                    key: key.as_int(),
                    channel: msg.channel,
                });
            }
            _ => {}
        }

        let msg = to_live_midi_event(msg);
        self.buf.clear();
        msg.write(&mut self.buf).unwrap();

        self.conn.send(&self.buf).ok();
    }

    fn stop_all(&mut self) {
        for note in std::mem::take(&mut self.active_notes).iter() {
            let msg = LiveEvent::Midi {
                channel: u4::new(note.channel),
                message: midly::MidiMessage::NoteOff {
                    key: u7::new(note.key),
                    vel: u7::new(0),
                },
            };
            self.buf.clear();
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

fn to_live_midi_event(msg: &midi_file::MidiEvent) -> midly::live::LiveEvent {
    midly::live::LiveEvent::Midi {
        channel: u4::new(msg.channel),
        message: msg.message,
    }
}
