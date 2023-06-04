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
    fn midi_event(&mut self, msg: &lib_midi::MidiEvent) {
        use lib_midi::midly::MidiMessage;
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

        let msg = libmidi_to_midi_event(msg);
        if let Some(msg) = msg.to_raw_messages().first() {
            match *msg {
                midi::RawMessage::StatusData(a, b) => {
                    self.conn.send(&[a, b]).ok();
                }
                midi::RawMessage::StatusDataData(a, b, c) => {
                    self.conn.send(&[a, b, c]).ok();
                }
                _ => {}
            }
        }
    }

    fn stop_all(&mut self) {
        for note in std::mem::take(&mut self.active_notes).iter() {
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

fn libmidi_to_midi_event(msg: &lib_midi::MidiEvent) -> midi::Message {
    use lib_midi::midly;

    let ch = midi::Channel::from_u8(msg.channel).unwrap();
    match msg.message {
        midly::MidiMessage::NoteOff { key, vel } => {
            midi::Message::NoteOff(ch, key.as_int(), vel.as_int())
        }
        midly::MidiMessage::NoteOn { key, vel } => {
            midi::Message::NoteOn(ch, key.as_int(), vel.as_int())
        }
        midly::MidiMessage::Aftertouch { key, vel } => {
            midi::Message::PolyphonicPressure(ch, key.as_int(), vel.as_int())
        }
        midly::MidiMessage::Controller { controller, value } => {
            midi::Message::ControlChange(ch, controller.as_int(), value.as_int())
        }
        midly::MidiMessage::ProgramChange { program } => {
            midi::Message::ProgramChange(ch, program.as_int())
        }
        midly::MidiMessage::ChannelAftertouch { vel } => {
            midi::Message::ChannelPressure(ch, vel.as_int())
        }
        midly::MidiMessage::PitchBend { bend } => midi::Message::PitchBend(ch, bend.0.as_int()),
    }
}
