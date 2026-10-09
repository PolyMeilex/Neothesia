use std::time::Duration;

use midi_file::MidiFile;

#[derive(Debug, Clone)]
pub struct Lyric {
    pub timestamp: Duration,
    pub text: String,
    pub width: f32,
    /// Position of the syllable start on the packed strip
    pub x: f32,
    /// Line (row) the syllable is rendered on, used for overlapping voices (duets, backing vocals)
    pub line: usize,
}

pub struct LyricsTrack {
    pub lyrics: Vec<Lyric>,
    pub font_size: f32,
    pub gap_width: f32,
}

impl LyricsTrack {
    fn calc_w(
        font_system: &mut cosmic_text::FontSystem,
        buf: &mut cosmic_text::Buffer,
        text: &str,
    ) -> f32 {
        buf.set_text(
            text,
            &cosmic_text::Attrs::new(),
            cosmic_text::Shaping::Basic,
            None,
        );
        buf.shape_until_scroll(font_system, false);
        buf.layout_runs().map(|r| r.line_w).fold(0.0, f32::max)
    }

    pub fn build(midi_file: &MidiFile) -> Self {
        let font_size = 32.0;
        let font_system = neothesia_core::font_system::font_system();
        let font_system = &mut *font_system.borrow_mut();

        let mut buf =
            cosmic_text::Buffer::new(font_system, cosmic_text::Metrics::new(font_size, font_size));

        let gap_width = Self::calc_w(font_system, &mut buf, " ");

        let mut lyrics = Vec::new();

        for track in midi_file.tracks.iter() {
            for sylable in track.lyrics.iter() {
                let width = Self::calc_w(font_system, &mut buf, &sylable.text);

                lyrics.push(Lyric {
                    timestamp: sylable.timestamp,
                    text: sylable.text.clone(),
                    width,
                    x: 0.0,
                    line: 0,
                });
            }
        }

        lyrics.sort_by_key(|l| l.timestamp);

        // Silence (in seconds) between syllable starts that still counts as continuous singing
        const SILENCE_THRESHOLD: f32 = 0.6;
        // Silence beyond this doesn't widen the gap any further
        const MAX_SILENCE: f32 = 3.0;
        // Silence after which lyrics go back to the first line
        const LINE_RESET_SILENCE: f32 = 1.0;

        let group_len = |lyrics: &[Lyric]| {
            let timestamp = lyrics[0].timestamp;
            lyrics
                .iter()
                .take_while(|l| l.timestamp == timestamp)
                .count()
        };

        // Syllables sharing a timestamp are sung by different voices, so each one needs its own line
        let mut lines = 1;
        let mut start = 0;
        while start < lyrics.len() {
            let len = group_len(&lyrics[start..]);
            lines = lines.max(len);
            start += len;
        }

        let mut line = 0;
        let mut x = 0.0;
        let mut start = 0;
        while start < lyrics.len() {
            let len = group_len(&lyrics[start..]);
            let timestamp = lyrics[start].timestamp;

            if let Some(prev) = start.checked_sub(1) {
                let dt = (timestamp - lyrics[prev].timestamp).as_secs_f32();
                if dt >= LINE_RESET_SILENCE {
                    line = 0;
                }
            }

            // The first voice stays on the current line, the rest move to the following ones.
            // Singing continues on the line of the last voice, like a new verse in sheet music.
            let group = &mut lyrics[start..start + len];
            for (k, lyric) in group.iter_mut().enumerate() {
                lyric.line = (line + k) % lines;
                lyric.x = x;
            }
            line = group[len - 1].line;

            // Stacked syllables take as much space as the widest one
            let width = group.iter().map(|l| l.width).fold(0.0, f32::max);
            x += width + gap_width;

            // Longer pauses get proportionally wider gaps, so phrases are visually separated
            if let Some(next) = lyrics.get(start + len) {
                let dt = (next.timestamp - timestamp).as_secs_f32();
                let silence = (dt - SILENCE_THRESHOLD).clamp(0.0, MAX_SILENCE);
                x += silence * font_size;
            }

            start += len;
        }

        Self {
            lyrics,
            font_size,
            gap_width,
        }
    }

    /// Strip scroll offset at `time`, such that every syllable's `x` is reached exactly at its timestamp.
    ///
    /// Between two syllables the strip moves linearly, so the speed varies per segment.
    /// Before the first and after the last syllable it moves at `edge_speed` (px/s).
    /// `t` is in seconds, relative to the song start (without lead-in), so it can be negative.
    pub fn strip_offset(&self, t: f32, edge_speed: f32) -> f32 {
        let next = self
            .lyrics
            .partition_point(|l| l.timestamp.as_secs_f32() <= t);
        let prev = next.checked_sub(1).map(|i| &self.lyrics[i]);

        match (prev, self.lyrics.get(next)) {
            (Some(a), Some(b)) => {
                let ta = a.timestamp.as_secs_f32();
                let tb = b.timestamp.as_secs_f32();
                // tb > ta is guaranteed by the partition point. Stacked syllables share `x`,
                // so jumping over the rest of a group doesn't move the strip
                let p = (t - ta) / (tb - ta);
                a.x + (b.x - a.x) * p
            }
            (None, Some(b)) => b.x - (b.timestamp.as_secs_f32() - t) * edge_speed,
            (Some(a), None) => a.x + (t - a.timestamp.as_secs_f32()) * edge_speed,
            (None, None) => 0.0,
        }
    }
}
