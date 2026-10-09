use std::time::Duration;

use midi_file::MidiFile;

#[derive(Debug, Clone)]
pub struct Lyric {
    pub timestamp: Duration,
    pub text: String,
    pub width: f32,
    /// Position of the syllable start on the packed strip
    pub x: f32,
    /// Time (in seconds) at which `x` crosses the marker.
    /// Same as `timestamp`, except for syllables that share a timestamp, see [`LyricsTrack::spread_scroll_times`]
    pub scroll_time: f32,
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
                    scroll_time: sylable.timestamp.as_secs_f32(),
                });
            }
        }

        lyrics.sort_by_key(|l| l.timestamp);

        // Silence (in seconds) between syllable starts that still counts as continuous singing
        const SILENCE_THRESHOLD: f32 = 0.6;
        // Silence beyond this doesn't widen the gap any further
        const MAX_SILENCE: f32 = 3.0;

        let mut x = 0.0;
        for i in 0..lyrics.len() {
            lyrics[i].x = x;
            x += lyrics[i].width + gap_width;

            // Longer pauses get proportionally wider gaps, so phrases are visually separated
            if let Some(next) = lyrics.get(i + 1) {
                let dt = (next.timestamp - lyrics[i].timestamp).as_secs_f32();
                let silence = (dt - SILENCE_THRESHOLD).clamp(0.0, MAX_SILENCE);
                x += silence * font_size;
            }
        }

        Self::spread_scroll_times(&mut lyrics);

        Self {
            lyrics,
            font_size,
            gap_width,
        }
    }

    /// Two syllables can't cross the marker at the same time, so for every group of syllables
    /// that share a timestamp, pull the earlier ones back in time. The last one stays on time,
    /// the rest are spread over a short window before it, which keeps `scroll_time` strictly increasing.
    fn spread_scroll_times(lyrics: &mut [Lyric]) {
        const MAX_WINDOW: f32 = 0.15;

        let mut prev_time: Option<f32> = None;
        let mut start = 0;

        while start < lyrics.len() {
            let time = lyrics[start].scroll_time;
            let len = lyrics[start..]
                .iter()
                .take_while(|l| l.scroll_time == time)
                .count();

            if len > 1 {
                // Never reach back further than half way to the previous syllable
                let window = match prev_time {
                    Some(prev) => MAX_WINDOW.min((time - prev) / 2.0),
                    None => MAX_WINDOW,
                };

                let last = len - 1;
                for (i, lyric) in lyrics[start..start + len].iter_mut().enumerate() {
                    lyric.scroll_time = time - window * (last - i) as f32 / last as f32;
                }
            }

            prev_time = Some(time);
            start += len;
        }
    }

    /// Strip scroll offset at `time`, such that every syllable's `x` is reached exactly at its `scroll_time`.
    ///
    /// Between two syllables the strip moves linearly, so the speed varies per segment.
    /// Before the first and after the last syllable it moves at `edge_speed` (px/s).
    /// `t` is in seconds, relative to the song start (without lead-in), so it can be negative.
    pub fn strip_offset(&self, t: f32, edge_speed: f32) -> f32 {
        let next = self.lyrics.partition_point(|l| l.scroll_time <= t);
        let prev = next.checked_sub(1).map(|i| &self.lyrics[i]);

        match (prev, self.lyrics.get(next)) {
            (Some(a), Some(b)) => {
                let ta = a.scroll_time;
                let tb = b.scroll_time;
                // tb > ta is guaranteed by the partition point
                let p = (t - ta) / (tb - ta);
                a.x + (b.x - a.x) * p
            }
            (None, Some(b)) => b.x - (b.scroll_time - t) * edge_speed,
            (Some(a), None) => a.x + (t - a.scroll_time) * edge_speed,
            (None, None) => 0.0,
        }
    }
}
