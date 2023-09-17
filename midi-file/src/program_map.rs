use crate::MidiEvent;
use std::{collections::HashMap, sync::OnceLock, time::Duration};

/// HashMap<Channel, Program>
fn default_programs() -> &'static HashMap<u8, u8> {
    static DEFAULT_PROGRAMS: OnceLock<HashMap<u8, u8>> = OnceLock::new();
    DEFAULT_PROGRAMS.get_or_init(|| (0..16).map(|ch| (ch, 0)).collect())
}

#[derive(Debug, Clone)]
struct Bucket {
    timestamp: Duration,
    map: HashMap<u8, u8>,
}

#[derive(Debug, Clone)]
pub struct ProgramMap {
    timestamps: Vec<Bucket>,
}

impl ProgramMap {
    pub fn new(events: &[MidiEvent]) -> Self {
        let mut map = default_programs().clone();

        let mut timestamps = Vec::new();

        for event in events {
            if let midly::MidiMessage::ProgramChange { program } = event.message {
                *map.entry(event.channel).or_default() = program.as_int();

                timestamps.push(Bucket {
                    timestamp: event.timestamp,
                    map: map.clone(),
                });
            }
        }

        Self { timestamps }
    }

    /// Search for program at certain timestamp
    pub fn program_for_timestamp(&self, timestamp: &Duration) -> &HashMap<u8, u8> {
        let res = self
            .timestamps
            .binary_search_by_key(timestamp, |bucket| bucket.timestamp);

        let id = match res {
            Ok(id) => Some(id),
            Err(id) => id.checked_sub(1),
        };

        id.map(|id| &self.timestamps[id].map)
            .unwrap_or_else(|| default_programs())
    }
}
