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
