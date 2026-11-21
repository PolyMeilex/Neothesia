use std::time::Duration;

use midi_file::LyricLine;
use neothesia_core::render::{QuadInstance, QuadRenderer, TextRenderer};

const SUNG: cosmic_text::Color = cosmic_text::Color::rgb(255, 210, 64);
const UNSUNG: cosmic_text::Color = cosmic_text::Color::rgb(255, 255, 255);
const NEXT: cosmic_text::Color = cosmic_text::Color::rgb(150, 150, 150);

const CURRENT_SIZE: f32 = 32.0;
const NEXT_SIZE: f32 = 22.0;
/// Gap between the top bar and the lyrics box
const MARGIN_TOP: f32 = 20.0;
const PADDING_X: f32 = 20.0;
const PADDING_Y: f32 = 10.0;
const LINE_SPACING: f32 = 8.0;

/// Show a line this long before its first syllable
const PREVIEW: Duration = Duration::from_millis(600);
/// Hide the current line this long after its last syllable, unless the next one is close
const LINGER: Duration = Duration::from_secs(3);
/// Show the upcoming line only when it starts within this time
const NEXT_WINDOW: Duration = Duration::from_secs(8);

/// Karaoke style lyrics displayed on top of the waterfall
pub struct LyricsView;

impl LyricsView {
    pub fn update(
        lines: &[LyricLine],
        time: f32,
        window_width: f32,
        top_bar_bottom: f32,
        quad_renderer: &mut QuadRenderer,
        text_renderer: &mut TextRenderer,
    ) {
        if lines.is_empty() || time < 0.0 {
            return;
        }
        let time = Duration::from_secs_f32(time);
        let top = top_bar_bottom + MARGIN_TOP;

        let Some((current, next)) = Self::visible_lines(lines, time) else {
            return;
        };

        let current_buffer = Self::line_buffer(current, time, CURRENT_SIZE, false);
        let next_buffer = next.map(|next| Self::line_buffer(next, time, NEXT_SIZE, true));

        let (current_w, current_h) = TextRenderer::measure(&current_buffer);
        let (next_w, next_h) = next_buffer
            .as_ref()
            .map(TextRenderer::measure)
            .unwrap_or((0.0, 0.0));

        let content_w = current_w.max(next_w);
        let content_h = current_h
            + if next_buffer.is_some() {
                LINE_SPACING + next_h
            } else {
                0.0
            };

        quad_renderer.push(QuadInstance {
            position: [
                window_width / 2.0 - content_w / 2.0 - PADDING_X,
                top - PADDING_Y,
            ],
            size: [content_w + PADDING_X * 2.0, content_h + PADDING_Y * 2.0],
            color: wgpu_jumpstart::Color::new(0.0, 0.0, 0.0, 0.6).into_linear_rgba(),
            border_radius: [10.0; 4],
            ..Default::default()
        });

        text_renderer.queue_buffer(window_width / 2.0 - current_w / 2.0, top, current_buffer);
        if let Some(next_buffer) = next_buffer {
            text_renderer.queue_buffer(
                window_width / 2.0 - next_w / 2.0,
                top + current_h + LINE_SPACING,
                next_buffer,
            );
        }
    }

    /// Pick the line being sung now and the one after it
    fn visible_lines(
        lines: &[LyricLine],
        time: Duration,
    ) -> Option<(&LyricLine, Option<&LyricLine>)> {
        let upcoming = lines.partition_point(|line| line.start <= time + PREVIEW);

        let next_within_window = |id: usize| {
            lines
                .get(id)
                .filter(|line| line.start.saturating_sub(time) < NEXT_WINDOW)
        };

        if upcoming == 0 {
            // Before the first line: show it shortly before it starts, not highlighted yet
            let first = &lines[0];
            return (first.start.saturating_sub(time) < LINGER).then_some((first, None));
        }

        let current = &lines[upcoming - 1];
        let next = next_within_window(upcoming);

        let finished = time.saturating_sub(current.end()) > LINGER;
        if finished {
            // Instrumental gap: announce the next line a bit earlier
            return next
                .filter(|next| next.start.saturating_sub(time) < LINGER)
                .map(|next| (next, None));
        }

        Some((current, next))
    }

    fn line_buffer(
        line: &LyricLine,
        time: Duration,
        size: f32,
        is_next: bool,
    ) -> cosmic_text::Buffer {
        let font_system = neothesia_core::font_system::font_system();
        let font_system = &mut font_system.borrow_mut();

        let spans: Vec<(String, cosmic_text::Color)> = line
            .syllables
            .iter()
            .map(|syllable| {
                let mut text = syllable.text.clone();
                if !syllable.joins_next {
                    text.push(' ');
                }

                let color = if is_next {
                    NEXT
                } else if syllable.timestamp <= time {
                    SUNG
                } else {
                    UNSUNG
                };

                (text, color)
            })
            .collect();

        let attrs = cosmic_text::Attrs::new().family(cosmic_text::Family::Name("Roboto"));

        let mut buffer =
            cosmic_text::Buffer::new(font_system, cosmic_text::Metrics::new(size, size * 1.2));
        buffer.set_size(Some(f32::MAX), Some(f32::MAX));
        buffer.set_rich_text(
            spans
                .iter()
                .map(|(text, color)| (text.as_str(), attrs.clone().color(*color))),
            &attrs,
            cosmic_text::Shaping::Advanced,
            None,
        );
        buffer.shape_until_scroll(font_system, false);
        buffer
    }
}
