use std::collections::HashSet;

use crate::output_manager::{OutputConnection, OutputDescriptor};

use lib_midi::ActiveNote;
use midi::ToRawMessages;
use num::FromPrimitive;

pub struct MidiOutputConnection {
    conn: midi_io::MidiOutputConnection,
    active_notes: HashSet<ActiveNote>,
}

impl From<midi_io::MidiOutputConnection> for MidiOutputConnection {
    fn from(conn: midi_io::MidiOutputConnection) -> Self {
        Self {
            conn,
            active_notes: Default::default(),
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
    fn midi_event(&mut self, msg: midi::Message) {
        match &msg {
            midi::Message::NoteOff(ch, key, _) => {
                let channel = channel_to_u8(ch);
                self.active_notes.remove(&ActiveNote { key: *key, channel });
            }
            midi::Message::NoteOn(ch, key, _) => {
                let channel = channel_to_u8(ch);
                self.active_notes.insert(ActiveNote { key: *key, channel });
            }
            _ => {}
        }

        if let Some(msg) = msg.to_raw_messages().first() {
            match *msg {
                midi::RawMessage::StatusDataData(a, b, c) => {
                    self.conn.send(&[a, b, c]).ok();
                }
                _ => {}
            }
        }
    }
}

impl Drop for MidiOutputConnection {
    fn drop(&mut self) {
        for note in self.active_notes.iter() {
            use midi::utils::{mask7, status_byte};

            let sb = status_byte(
                midi::constants::NOTE_OFF,
                midi::Channel::from_u8(note.channel).unwrap(),
            );
            let data = [sb, mask7(note.key), mask7(0)];
            self.conn.send(&data).ok();
        }
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

fn channel_to_u8(ch: &midi::Channel) -> u8 {
    match ch {
        midi::Channel::Ch1 => 0,
        midi::Channel::Ch2 => 1,
        midi::Channel::Ch3 => 2,
        midi::Channel::Ch4 => 3,
        midi::Channel::Ch5 => 4,
        midi::Channel::Ch6 => 5,
        midi::Channel::Ch7 => 6,
        midi::Channel::Ch8 => 7,
        midi::Channel::Ch9 => 8,
        midi::Channel::Ch10 => 9,
        midi::Channel::Ch11 => 10,
        midi::Channel::Ch12 => 11,
        midi::Channel::Ch13 => 12,
        midi::Channel::Ch14 => 13,
        midi::Channel::Ch15 => 14,
        midi::Channel::Ch16 => 15,
    }
}
