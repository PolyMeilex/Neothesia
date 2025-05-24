use midi_file::MidiNote;
use midi_file::MidiTrack;

use super::KeyboardRenderer;
use super::TextRenderer;
use super::TextRendererInstance;

#[derive(Default)]
struct LabelsCache {
    labels: Option<[glyphon::Buffer; 12]>,
    neutral_width: f32,
}

impl LabelsCache {
    fn get(
        &mut self,
        keyboard: &KeyboardRenderer,
        font_system: &mut glyphon::FontSystem,
    ) -> &[glyphon::Buffer; 12] {
        let sharp_width = keyboard.layout().sizing.sharp_width;
        let neutral_width = keyboard.layout().sizing.neutral_width;

        if self.labels.is_none() || self.neutral_width != neutral_width {
            let label_width = sharp_width;

            let labels = [
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
                let mut buffer = glyphon::Buffer::new(
                    font_system,
                    glyphon::Metrics::new(label_width, label_width),
                );
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
            });

            self.labels = Some(labels);
        }

        self.labels.as_ref().unwrap()
    }
}

pub struct NoteLabels {
    notes: Box<[MidiNote]>,
    labels_cache: LabelsCache,
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
            labels_cache: LabelsCache::default(),
        }
    }

    #[profiling::function]
    pub fn update(
        &mut self,
        text: &mut TextRenderer,
        text_instance: &mut TextRendererInstance,
        keyboard: &KeyboardRenderer,
        animation_speed: f32,
        time: f32,
    ) {
        let range_start = keyboard.range().start() as usize;
        let full_height = keyboard.pos().y + keyboard.layout().height;
        let layout = keyboard.layout();
        let label_width = layout.sizing.sharp_width;

        let labels = self.labels_cache.get(keyboard, text.font_system());

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

            text_instance.queue(super::text::TextArea {
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
