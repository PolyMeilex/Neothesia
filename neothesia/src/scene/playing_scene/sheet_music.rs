use std::time::Duration;

use midi_file::{MidiNote, MidiTrack};
use neothesia_core::{
    Rect,
    render::{QuadInstance, QuadRenderer},
};
use wgpu_jumpstart::Color;
use winit::event::WindowEvent;

use crate::{context::Context, utils::window::WinitEvent};

mod sprite;
use sprite::{SpriteKind, SpriteRenderer};

const PANEL_TOP: f32 = 82.0;
const MIN_HEIGHT: f32 = 140.0;
const PIXELS_PER_SECOND: f32 = 180.0;

#[derive(Clone, Copy)]
struct ScoreNote {
    start: f32,
    duration: f32,
    key: u8,
    color_id: usize,
}

impl From<&MidiNote> for ScoreNote {
    fn from(note: &MidiNote) -> Self {
        Self {
            start: note.start.as_secs_f32(),
            duration: note.duration.as_secs_f32(),
            key: note.note,
            color_id: note.track_color_id,
        }
    }
}

pub struct SheetMusicRenderer {
    notes: Vec<ScoreNote>,
    measures: Box<[f32]>,
    quarter_note_duration: f32,
    quads: QuadRenderer,
    sprites: SpriteRenderer,
    overlay: QuadRenderer,
    resizing: bool,
}

impl SheetMusicRenderer {
    pub fn new(
        ctx: &Context,
        tracks: &[MidiTrack],
        hidden_tracks: &[usize],
        measures: &[Duration],
    ) -> Self {
        let mut notes: Vec<_> = tracks
            .iter()
            .filter(|track| {
                !hidden_tracks.contains(&track.track_id)
                    && !(track.has_drums && !track.has_other_than_drums)
            })
            .flat_map(|track| track.notes.iter().map(ScoreNote::from))
            .collect();
        notes.sort_by(|a, b| a.start.total_cmp(&b.start));

        let measures: Box<[f32]> = measures
            .iter()
            .map(Duration::as_secs_f32)
            .collect::<Vec<_>>()
            .into();
        let quarter_note_duration = measures
            .windows(2)
            .find_map(|pair| {
                let duration = pair[1] - pair[0];
                (duration > 0.0).then_some(duration / 4.0)
            })
            .unwrap_or(0.5);

        Self {
            notes,
            measures,
            quarter_note_duration,
            quads: ctx.quad_renderer_facotry.new_renderer(),
            sprites: SpriteRenderer::new(ctx),
            overlay: ctx.quad_renderer_facotry.new_renderer(),
            resizing: false,
        }
    }

    pub fn update(&mut self, ctx: &Context, current_time: f32) {
        self.quads.clear();
        self.sprites.clear();
        self.overlay.clear();

        let width = ctx.window_state.logical_size.width;
        let height = self.height(ctx);
        let scale = ctx.window_state.scale_factor as f32;
        let panel_rect = Rect::new(
            ((0.0 * scale) as u32, (PANEL_TOP * scale) as u32).into(),
            ((width * scale) as u32, (height * scale) as u32).into(),
        );
        self.quads.set_scissor_rect(panel_rect);
        self.sprites.set_scissor_rect(panel_rect);
        self.overlay.set_scissor_rect(panel_rect);

        self.push_quad(0.0, PANEL_TOP, width, height, rgb(8, 8, 11, 0.96), 0.0);

        let gap = ((height - 42.0) / 13.0).clamp(7.0, 17.0);
        let grand_staff_height = gap * 12.0;
        let treble_top = PANEL_TOP + (height - grand_staff_height) / 2.0;
        let bass_top = treble_top + gap * 8.0;
        let staff_color = rgb(205, 205, 215, 0.72);

        for line in 0..5 {
            let offset = line as f32 * gap;
            self.push_quad(0.0, treble_top + offset, width, 1.0, staff_color, 0.0);
            self.push_quad(0.0, bass_top + offset, width, 1.0, staff_color, 0.0);
        }

        let min_time = current_time - width / (2.0 * PIXELS_PER_SECOND) - 0.1;
        let max_time = current_time + width / (2.0 * PIXELS_PER_SECOND) + 0.1;

        let first_measure = self.measures.partition_point(|&time| time < min_time);
        let mut measure_index = first_measure;
        while measure_index < self.measures.len() {
            let measure = self.measures[measure_index];
            if measure > max_time {
                break;
            }
            let x = time_to_x(measure, current_time, width, PIXELS_PER_SECOND);
            self.push_quad(
                x,
                treble_top,
                1.0,
                bass_top + gap * 4.0 - treble_top,
                rgb(155, 155, 165, 0.55),
                0.0,
            );
            measure_index += 1;
        }

        let first_note = self.notes.partition_point(|note| note.start < min_time);
        let visible_notes = self.notes[first_note..]
            .iter()
            .take_while(|note| note.start <= max_time)
            .copied()
            .collect::<Vec<_>>();

        for note in visible_notes {
            let x = time_to_x(note.start, current_time, width, PIXELS_PER_SECOND);
            let (bottom_line, staff_step) = if note.key >= 60 {
                (
                    treble_top + gap * 4.0,
                    diatonic_index(note.key) - diatonic_index(64),
                )
            } else {
                (
                    bass_top + gap * 4.0,
                    diatonic_index(note.key) - diatonic_index(43),
                )
            };
            let y = bottom_line - staff_step as f32 * gap / 2.0;

            self.draw_ledger_lines(x, bottom_line, staff_step, gap);
            self.draw_note(ctx, note, x, y, gap);
        }

        self.draw_clefs(treble_top, bass_top, gap);

        // The playback cursor never moves; the score moves around it.
        self.push_overlay(
            width / 2.0 - 1.0,
            PANEL_TOP,
            2.0,
            height,
            rgb(255, 82, 105, 0.95),
            0.0,
        );

        let handle_color = if self.resizing {
            rgb(255, 255, 255, 0.95)
        } else {
            rgb(125, 125, 138, 0.9)
        };
        self.push_overlay(
            width / 2.0 - 28.0,
            PANEL_TOP + height - 5.0,
            56.0,
            3.0,
            handle_color,
            2.0,
        );

        self.quads.prepare();
        self.sprites.prepare();
        self.overlay.prepare();
    }

    pub fn render<'pass>(&'pass self, rpass: &mut wgpu_jumpstart::RenderPass<'pass>) {
        self.quads.render(rpass);
        self.sprites.render(rpass);
        self.overlay.render(rpass);
    }

    pub fn handle_window_event(&mut self, ctx: &mut Context, event: &WindowEvent) {
        let cursor_y = ctx.window_state.cursor_logical_position.y;
        let bottom = PANEL_TOP + self.height(ctx);

        if event.left_mouse_pressed() && (cursor_y - bottom).abs() <= 10.0 {
            self.resizing = true;
        }

        if event.left_mouse_released() {
            self.resizing = false;
        }

        if self.resizing && (event.cursor_moved() || event.left_mouse_pressed()) {
            let max_height = self.max_height(ctx);
            ctx.config
                .set_sheet_music_height((cursor_y - PANEL_TOP).clamp(MIN_HEIGHT, max_height));
        }
    }

    fn height(&self, ctx: &Context) -> f32 {
        ctx.config
            .sheet_music_height()
            .clamp(MIN_HEIGHT, self.max_height(ctx))
    }

    fn max_height(&self, ctx: &Context) -> f32 {
        // The keyboard occupies the bottom 20% of the window.
        (ctx.window_state.logical_size.height * 0.8 - PANEL_TOP - 8.0).max(MIN_HEIGHT)
    }

    fn draw_note(&mut self, ctx: &Context, note: ScoreNote, x: f32, y: f32, gap: f32) {
        let color = ctx
            .config
            .color_schema()
            .get(note.color_id % ctx.config.color_schema().len().max(1))
            .map(|color| rgb(color.base.0, color.base.1, color.base.2, 1.0))
            .unwrap_or_else(|| rgb(235, 235, 240, 1.0));

        let is_half = note.duration >= self.quarter_note_duration * 1.75;
        let is_whole = note.duration >= self.quarter_note_duration * 3.5;

        let (kind, sprite_height, anchor_y) = if is_whole {
            (SpriteKind::WholeNote, gap * 5.0, 0.5)
        } else if is_half {
            (SpriteKind::HalfNote, gap * 7.0, 0.62)
        } else {
            (SpriteKind::QuarterNote, gap * 7.0, 0.70)
        };
        let sprite_width = sprite_height * (2.0 / 3.0);
        self.sprites.push(
            kind,
            [x - sprite_width * 0.5, y - sprite_height * anchor_y],
            [sprite_width, sprite_height],
            color,
        );

        if is_accidental(note.key) {
            let sharp_height = gap * 4.0;
            let sharp_width = sharp_height * (2.0 / 3.0);
            self.sprites.push(
                SpriteKind::Sharp,
                [x - gap * 1.6 - sharp_width * 0.5, y - sharp_height * 0.5],
                [sharp_width, sharp_height],
                color,
            );
        }
    }

    fn draw_ledger_lines(&mut self, x: f32, bottom_line: f32, staff_step: i32, gap: f32) {
        let line_w = gap * 2.1;
        let color = rgb(205, 205, 215, 0.72);
        if staff_step < 0 {
            let mut step = -2;
            while step >= staff_step {
                let y = bottom_line - step as f32 * gap / 2.0;
                self.push_quad(x - line_w / 2.0, y, line_w, 1.0, color, 0.0);
                step -= 2;
            }
        } else if staff_step > 8 {
            let mut step = 10;
            while step <= staff_step {
                let y = bottom_line - step as f32 * gap / 2.0;
                self.push_quad(x - line_w / 2.0, y, line_w, 1.0, color, 0.0);
                step += 2;
            }
        }
    }

    fn draw_clefs(&mut self, treble_top: f32, bass_top: f32, gap: f32) {
        let color = rgb(245, 245, 250, 1.0);
        let treble_height = gap * 7.2;
        let treble_width = treble_height * (2.0 / 3.0);
        self.sprites.push(
            SpriteKind::TrebleClef,
            [gap * 0.2, treble_top + gap * 2.0 - treble_height * 0.48],
            [treble_width, treble_height],
            color,
        );

        let bass_height = gap * 6.0;
        let bass_width = bass_height * (2.0 / 3.0);
        self.sprites.push(
            SpriteKind::BassClef,
            [gap * 0.25, bass_top + gap * 2.0 - bass_height * 0.44],
            [bass_width, bass_height],
            color,
        );
    }

    fn push_quad(&mut self, x: f32, y: f32, width: f32, height: f32, color: [f32; 4], radius: f32) {
        self.quads.push(QuadInstance {
            position: [x, y],
            size: [width.max(0.0), height.max(0.0)],
            color,
            border_radius: [radius; 4],
        });
    }

    fn push_overlay(
        &mut self,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        color: [f32; 4],
        radius: f32,
    ) {
        self.overlay.push(QuadInstance {
            position: [x, y],
            size: [width.max(0.0), height.max(0.0)],
            color,
            border_radius: [radius; 4],
        });
    }
}

fn rgb(r: u8, g: u8, b: u8, alpha: f32) -> [f32; 4] {
    Color::from_rgba8(r, g, b, alpha).into_linear_rgba()
}

fn time_to_x(note_time: f32, current_time: f32, width: f32, pixels_per_second: f32) -> f32 {
    width / 2.0 + (note_time - current_time) * pixels_per_second
}

fn diatonic_index(midi_note: u8) -> i32 {
    let octave = midi_note as i32 / 12 - 1;
    let degree = match midi_note % 12 {
        0 | 1 => 0,  // C / C sharp
        2 | 3 => 1,  // D / D sharp
        4 => 2,      // E
        5 | 6 => 3,  // F / F sharp
        7 | 8 => 4,  // G / G sharp
        9 | 10 => 5, // A / A sharp
        11 => 6,     // B
        _ => unreachable!(),
    };
    octave * 7 + degree
}

fn is_accidental(midi_note: u8) -> bool {
    matches!(midi_note % 12, 1 | 3 | 6 | 8 | 10)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_playback_position_is_centered() {
        assert_eq!(time_to_x(12.5, 12.5, 1000.0, PIXELS_PER_SECOND), 500.0);
        assert!(time_to_x(13.0, 12.5, 1000.0, PIXELS_PER_SECOND) > 500.0);
        assert!(time_to_x(12.0, 12.5, 1000.0, PIXELS_PER_SECOND) < 500.0);
    }

    #[test]
    fn pitch_mapping_advances_by_staff_steps() {
        assert_eq!(diatonic_index(60), 28); // C4
        assert_eq!(diatonic_index(62), 29); // D4
        assert_eq!(diatonic_index(64), 30); // E4
        assert_eq!(diatonic_index(61), 28); // C sharp shares C's staff position
    }
}
