use std::time::Duration;

use cosmic_text::Buffer;
use midi_file::MidiFile;

#[derive(Debug, Clone)]
pub struct Lyric {
    pub timestamp: Duration,
    pub text: String,
    pub width: f32,
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
                });
            }
        }

        Self {
            lyrics,
            font_size,
            gap_width,
        }
    }
}
