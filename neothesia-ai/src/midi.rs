pub struct Note {
    pub pitch: u8,
    /// Seconds
    pub start: f64,
    /// Seconds
    pub end: f64,
    pub velocity: u8,
}

pub struct ControlChange {
    pub controller: u8,
    pub value: u8,
    /// Seconds
    pub time: f64,
}

pub fn create_midi_file(notes: &[Note], control_changes: &[ControlChange]) -> midly::Smf<'static> {
    let ticks_per_beat = 384;
    let beats_per_second = 2;
    let ticks_per_second = (ticks_per_beat * beats_per_second) as f64;
    let microseconds_per_beat = (1_000_000.0 / beats_per_second as f64) as u32;

    let to_ticks = |time: f64| (time * ticks_per_second).round().max(0.0) as u32;

    // (ticks, order, message), `order` makes sure that note-offs come before note-ons
    // (and pedal changes) that happen at the same tick
    let mut message_roll = vec![];

    for note in notes {
        message_roll.push((
            to_ticks(note.end),
            0,
            midly::MidiMessage::NoteOff {
                key: note.pitch.into(),
                vel: 0.into(),
            },
        ));
        message_roll.push((
            to_ticks(note.start),
            1,
            midly::MidiMessage::NoteOn {
                key: note.pitch.into(),
                vel: note.velocity.into(),
            },
        ));
    }

    for cc in control_changes {
        message_roll.push((
            to_ticks(cc.time),
            1,
            midly::MidiMessage::Controller {
                controller: cc.controller.into(),
                value: cc.value.into(),
            },
        ));
    }

    message_roll.sort_by_key(|(ticks, order, _)| (*ticks, *order));

    let mut track1 = vec![];
    let mut previous_ticks = 0;

    for (ticks, _, message) in message_roll {
        track1.push(midly::TrackEvent {
            delta: (ticks - previous_ticks).into(),
            kind: midly::TrackEventKind::Midi {
                channel: 0.into(),
                message,
            },
        });
        previous_ticks = ticks;
    }

    track1.push(midly::TrackEvent {
        delta: 1.into(),
        kind: midly::TrackEventKind::Meta(midly::MetaMessage::EndOfTrack),
    });

    midly::Smf {
        header: midly::Header {
            format: midly::Format::Parallel,
            timing: midly::Timing::Metrical(ticks_per_beat.into()),
        },
        tracks: vec![
            vec![
                midly::TrackEvent {
                    delta: 0.into(),
                    kind: midly::TrackEventKind::Meta(midly::MetaMessage::Tempo(
                        microseconds_per_beat.into(),
                    )),
                },
                midly::TrackEvent {
                    delta: 0.into(),
                    kind: midly::TrackEventKind::Meta(midly::MetaMessage::TimeSignature(
                        4, 2, 24, 8,
                    )),
                },
                midly::TrackEvent {
                    delta: 1.into(),
                    kind: midly::TrackEventKind::Meta(midly::MetaMessage::EndOfTrack),
                },
            ],
            track1,
        ],
    }
}
