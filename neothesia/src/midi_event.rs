use num::FromPrimitive;

#[derive(Clone, Copy, Debug)]
pub enum MidiEvent {
    NoteOn {
        channel: u8,
        track_id: usize,
        key: u8,
        vel: u8,
    },
    NoteOff {
        channel: u8,
        key: u8,
    },
}

impl From<MidiEvent> for midi::Message {
    fn from(from: MidiEvent) -> Self {
        match from {
            MidiEvent::NoteOn {
                channel, key, vel, ..
            } => midi::Message::NoteOn(midi::Channel::from_u8(channel).unwrap(), key, vel),
            MidiEvent::NoteOff { channel, key } => {
                midi::Message::NoteOff(midi::Channel::from_u8(channel).unwrap(), key, 0)
            }
        }
    }
}
