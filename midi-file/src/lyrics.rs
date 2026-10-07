use std::time::Duration;

use midly::{MetaMessage, Track, TrackEventKind};

use crate::tempo_track::TempoTrack;

/// A single sung syllable (one MIDI `Lyric` meta event)
#[derive(Debug, Clone)]
pub struct LyricSyllable {
    pub timestamp: Duration,
    pub text: String,
    /// Syllable is followed by the next one without a space (it ended with `-`)
    pub joins_next: bool,
}

/// A phrase of lyrics that is displayed at once
#[derive(Debug, Clone)]
pub struct LyricLine {
    pub start: Duration,
    pub syllables: Vec<LyricSyllable>,
}

impl LyricLine {
    pub fn end(&self) -> Duration {
        self.syllables
            .last()
            .map(|s| s.timestamp)
            .unwrap_or(self.start)
    }
}

/// Max silence between syllables before a new line is started (for files without line markers)
const LINE_GAP: Duration = Duration::from_millis(2500);
/// Soft limit of line length in characters (for files without line markers)
const LINE_CHARS: usize = 50;

/// MIDI text has no defined encoding: newer files use UTF-8, older ones mostly Latin-1
fn decode_text(bytes: &[u8]) -> String {
    match std::str::from_utf8(bytes) {
        Ok(text) => text.to_string(),
        Err(_) => bytes.iter().map(|&b| b as char).collect(),
    }
}

fn is_line_marker(c: char) -> bool {
    matches!(c, '/' | '\\' | '\r' | '\n')
}

/// Collect `Lyric` meta events from all tracks and group them into lines.
///
/// Supports the common karaoke conventions: a syllable starting with `/`, `\`, CR or LF begins
/// a new line, and a syllable ending with `-` is joined with the next one. Files without line
/// markers are split on long pauses and long lines.
pub fn build_lyrics(tracks: &[Track], tempo_track: &TempoTrack) -> Vec<LyricLine> {
    let mut raw: Vec<(u64, String)> = Vec::new();

    for track in tracks {
        let mut pulses: u64 = 0;
        for event in track {
            pulses += event.delta.as_int() as u64;
            if let TrackEventKind::Meta(MetaMessage::Lyric(bytes)) = event.kind {
                raw.push((pulses, decode_text(bytes)));
            }
        }
    }

    // Stable sort keeps the original order of syllables that share a timestamp
    raw.sort_by_key(|(pulses, _)| *pulses);

    let has_markers = raw
        .iter()
        .any(|(_, text)| text.starts_with(is_line_marker) || text.ends_with(['\r', '\n']));

    let mut lines: Vec<LyricLine> = Vec::new();
    let mut current: Vec<LyricSyllable> = Vec::new();
    let mut current_chars = 0;

    let mut flush = |current: &mut Vec<LyricSyllable>, current_chars: &mut usize| {
        if let Some(first) = current.first() {
            lines.push(LyricLine {
                start: first.timestamp,
                syllables: std::mem::take(current),
            });
        }
        *current_chars = 0;
    };

    for (pulses, text) in raw {
        let timestamp = tempo_track.pulses_to_duration(pulses);

        let starts_line = text.starts_with(is_line_marker);
        let ends_line = text.ends_with(['\r', '\n']);

        let text = text.trim_matches(|c: char| is_line_marker(c) || c.is_whitespace());
        let joins_next = text.ends_with('-');
        let text = text.trim_end_matches('-').to_string();

        let mut new_line = starts_line;
        if !has_markers && let Some(last) = current.last() {
            let long_pause = timestamp.saturating_sub(last.timestamp) > LINE_GAP;
            let sentence_end = last.text.ends_with(['.', '?', '!', ')']) || text.starts_with('(');
            let too_long = current_chars > LINE_CHARS && !last.joins_next;
            new_line |= long_pause || sentence_end || too_long;
        }

        if new_line {
            flush(&mut current, &mut current_chars);
        }

        if !text.is_empty() {
            current_chars += text.chars().count() + 1;
            current.push(LyricSyllable {
                timestamp,
                text,
                joins_next,
            });
        }

        if ends_line {
            flush(&mut current, &mut current_chars);
        }
    }

    flush(&mut current, &mut current_chars);

    lines
}
