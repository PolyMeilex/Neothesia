use crate::MidiTrack;
use std::{
    collections::HashMap,
    sync::{Arc, OnceLock},
    time::Duration,
};

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
pub struct ProgramTrack {
    events: Arc<[Bucket]>,
}

impl ProgramTrack {
    pub fn new(tracks: &[MidiTrack]) -> Self {
        let mut map = default_programs().clone();

        // This map will help us get rid of duplicate events
        let mut program_events: HashMap<Duration, Bucket> = HashMap::new();

        for track in tracks {
            for event in track.events.iter() {
                if let midly::MidiMessage::ProgramChange { program } = event.message {
                    *map.entry(event.channel).or_default() = program.as_int();

                    program_events.insert(
                        event.timestamp,
                        Bucket {
                            timestamp: event.timestamp,
                            map: map.clone(),
                        },
                    );
                }
            }
        }

        let mut program_events: Vec<_> = program_events.into_values().collect();
        program_events.sort_by_key(|e| e.timestamp);

        Self {
            events: program_events.into(),
        }
    }

    /// Search for program at certain timestamp
    pub fn program_for_timestamp(&self, timestamp: &Duration) -> &HashMap<u8, u8> {
        let res = self
            .events
            .binary_search_by_key(timestamp, |bucket| bucket.timestamp);

        let id = match res {
            Ok(id) => Some(id),
            Err(id) => id.checked_sub(1),
        };

        id.map(|id| &self.events[id].map)
            .unwrap_or_else(|| default_programs())
    }
}
