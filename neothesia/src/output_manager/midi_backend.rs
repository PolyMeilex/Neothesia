use std::{cell::RefCell, collections::HashSet, rc::Rc};

use crate::output_manager::OutputDescriptor;

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

struct MidiOutputConnectionInner {
    conn: midi_io::MidiOutputConnection,
    active_notes: HashSet<ActiveNote>,
    buf: Vec<u8>,
}

#[derive(Clone)]
pub struct MidiOutputConnection {
    inner: Rc<RefCell<MidiOutputConnectionInner>>,
}

impl From<midi_io::MidiOutputConnection> for MidiOutputConnection {
    fn from(conn: midi_io::MidiOutputConnection) -> Self {
        Self {
            inner: Rc::new(RefCell::new(MidiOutputConnectionInner {
                conn,
                active_notes: Default::default(),
                buf: Vec::with_capacity(8),
            })),
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

impl MidiOutputConnection {
    pub fn midi_event(&self, channel: u4, message: midly::MidiMessage) {
        let inner = &mut *self.inner.borrow_mut();
        match message {
            midly::MidiMessage::NoteOff { key, .. } => {
                inner.active_notes.remove(&ActiveNote { key, channel });
            }
            midly::MidiMessage::NoteOn { key, .. } => {
                inner.active_notes.insert(ActiveNote { key, channel });
            }
            _ => {}
        }

        inner.buf.clear();
        let msg = midly::live::LiveEvent::Midi { channel, message };
        msg.write(&mut inner.buf).unwrap();

        inner.conn.send(&inner.buf).ok();
    }

    pub fn stop_all(&self) {
        let inner = &mut *self.inner.borrow_mut();
        for note in std::mem::take(&mut inner.active_notes).iter() {
            inner.buf.clear();
            let msg = LiveEvent::Midi {
                channel: note.channel,
                message: midly::MidiMessage::NoteOff {
                    key: note.key,
                    vel: u7::new(0),
                },
            };
            msg.write(&mut inner.buf).unwrap();

            inner.conn.send(&inner.buf).ok();
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
