use midi_file::MidiNote;
use midi_file::MidiTrack;

use super::KeyboardRenderer;
use super::TextRenderer;

pub struct NoteLabels {
    notes: Box<[MidiNote]>,
}

impl NoteLabels {
    pub fn new(tracks: &[MidiTrack], hidden_tracks: &[usize]) -> Self {
        let mut notes: Vec<_> = tracks
            .iter()
            .filter(|track| !hidden_tracks.contains(&track.track_id))
            .flat_map(|track| track.notes.iter().cloned())
            .collect();
        // We want to render newer notes on top of old notes
        notes.sort_unstable_by_key(|note| note.start);

        Self {
            notes: notes.into(),
        }
    }

    #[profiling::function]
    fn build_labels(
        &self,
        keyboard: &KeyboardRenderer,
        font_system: &mut glyphon::FontSystem,
    ) -> [glyphon::Buffer; 12] {
        let sharp_width = keyboard.layout().sizing.sharp_width;
        let neutral_width = keyboard.layout().sizing.neutral_width;
        let label_width = sharp_width;

        [
            ("C", neutral_width),
            ("C#", sharp_width),
            ("D", neutral_width),
            ("D#", sharp_width),
            ("E", neutral_width),
            ("F", neutral_width),
            ("F#", sharp_width),
            ("G", neutral_width),
            ("G#", sharp_width),
            ("A", neutral_width),
            ("A#", sharp_width),
            ("B", neutral_width),
        ]
        .map(|(label, note_width)| {
            let mut buffer =
                glyphon::Buffer::new(font_system, glyphon::Metrics::new(label_width, label_width));
            buffer.set_size(font_system, Some(note_width), None);
            buffer.set_wrap(font_system, glyphon::Wrap::None);
            buffer.set_text(
                font_system,
                label,
                glyphon::Attrs::new().family(glyphon::Family::SansSerif),
                glyphon::Shaping::Basic,
            );
            buffer.lines[0].set_align(Some(glyphon::cosmic_text::Align::Center));
            buffer.shape_until_scroll(font_system, false);
            buffer
        })
    }

    #[profiling::function]
    pub fn update(
        &mut self,
        text: &mut TextRenderer,
        keyboard: &KeyboardRenderer,
        animation_speed: f32,
        time: f32,
    ) {
        let range_start = keyboard.range().start() as usize;
        let full_height = keyboard.pos().y + keyboard.layout().height;
        let layout = keyboard.layout();
        let label_width = layout.sizing.sharp_width;

        let labels = self.build_labels(keyboard, text.font_system());

        for note in self.notes.iter() {
            let x = layout.keys[note.note as usize - range_start].x();
            let label_buffer = &labels[(note.note % 12) as usize];

            let y = (note.start.as_secs_f32() - time) * animation_speed
                + keyboard.layout().height
                + label_width;

            if y > full_height {
                break;
            }

            if full_height - y > keyboard.pos().y {
                continue;
            }

            text.queue(super::text::TextArea {
                buffer: label_buffer.clone(),
                left: x,
                top: full_height - y,
                scale: 1.0,
                bounds: glyphon::TextBounds {
                    left: 0,
                    top: 0,
                    right: i32::MAX,
                    bottom: i32::MAX,
                },
                default_color: glyphon::Color::rgb(255, 255, 255),
            });
        }
    }
}
