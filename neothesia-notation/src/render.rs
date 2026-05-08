use std::time::Duration;

use neothesia_core::render::{QuadInstance, QuadRenderer};
use wgpu_jumpstart::Color;

use neothesia_core::config::ColorSchemaV1;

use crate::{
    Chord, MIDDLE_LINE_POS, Notehead, Rest, RhythmicValue, Score, Staff, StaffElement, StaffPos,
    StaffRange, StemDirection,
};

/// Diatonic position of the top staff line (5 lines × 2 diatonic steps - 2).
const TOP_LINE_POS: StaffPos = 8;

// ── Base layout constants ──────────────────────────────────────────────

/// Stem height as a multiple of the staff line spacing.
const STEM_HEIGHT_LINES: f32 = 3.5;

const BASE_LINE_SPACING: f32 = 12.0;
const BASE_NOTE_HEAD_WIDTH: f32 = 14.0;
const BASE_NOTE_HEAD_HEIGHT: f32 = 10.5;
const BASE_STEM_HEIGHT: f32 = BASE_LINE_SPACING * STEM_HEIGHT_LINES;
const BASE_STEM_THICKNESS: f32 = 2.0;

/// How far past the visible viewport edges the staff lines are drawn,
/// so the lines are still visible during scrolling.
const STAFF_LINE_OVERDRAW: f32 = 200.0;

/// Approximate seconds of music visible across the viewport.
/// Smaller = more zoom (less time per screen).
const TARGET_SECONDS_VISIBLE: f32 = 14.0;
const MIN_PIXELS_PER_SEC: f32 = 50.0;
const MAX_PIXELS_PER_SEC: f32 = 600.0;

// ── Colors ─────────────────────────────────────────────────────────────

const PAPER_WHITE: Color = Color {
    r: 0.96,
    g: 0.95,
    b: 0.93,
    a: 1.0,
};
const INK_BLACK: Color = Color {
    r: 0.08,
    g: 0.08,
    b: 0.08,
    a: 1.0,
};
const STAFF_GRAY: Color = Color {
    r: 0.75,
    g: 0.75,
    b: 0.75,
    a: 1.0,
};
const PLAYHEAD_COLOR: Color = Color {
    r: 0.85,
    g: 0.25,
    b: 0.25,
    a: 1.0,
};
const PLAYHEAD_WIDTH: f32 = 2.0;

/// Pull the per-track note color from the schema, defaulting to ink black
/// when no schema is configured.
fn track_color(color_id: usize, schema: &[ColorSchemaV1]) -> Color {
    if schema.is_empty() {
        return INK_BLACK;
    }
    let entry = &schema[color_id % schema.len()];
    Color {
        r: entry.base.0 as f32 / 255.0,
        g: entry.base.1 as f32 / 255.0,
        b: entry.base.2 as f32 / 255.0,
        a: 1.0,
    }
}

// ── Public render args ────────────────────────────────────────────────

/// Where on screen the notation panel lives, plus its scroll offset.
pub struct RenderTarget {
    /// Window width in logical pixels.
    pub viewport_width: f32,
    /// Notation panel height in logical pixels.
    pub notation_height: f32,
    /// Notation-coord x at the left edge of the viewport. Negative scrolls
    /// the music to the right (used near song start to keep the playhead
    /// centered).
    pub scroll_x: f32,
    /// Y position where the notation panel starts (e.g. below the top bar).
    pub top_offset: f32,
}

// ── Per-render scale + context ─────────────────────────────────────────

/// Pixel sizes for one frame, derived from `notation_h` and the staff range.
#[derive(Clone, Copy)]
struct Scale {
    /// Vertical spacing between staff lines.
    ls: f32,
    /// Notehead width.
    nhw: f32,
    /// Notehead height.
    nhh: f32,
    /// Stem height past the end notehead.
    sh: f32,
    /// Stem thickness.
    st: f32,
}

/// Per-frame state shared by every drawing helper.
struct RenderCtx<'a> {
    scale: Scale,
    current_time: Duration,
    color_schema: &'a [ColorSchemaV1],
    /// Notation-coord x at the left edge of the viewport.
    scroll_x: f32,
    /// Notation-coord x bounds for visibility culling (with a small pad).
    vis_lo: f32,
    vis_hi: f32,
}

// ── Renderer ───────────────────────────────────────────────────────────

pub struct NotationRenderer {
    score: Score,
    pixels_per_sec: f32,
}

/// Pixels-per-second for the given viewport width, clamped to sane bounds.
fn pixels_per_sec_for(viewport_w: f32) -> f32 {
    (viewport_w / TARGET_SECONDS_VISIBLE).clamp(MIN_PIXELS_PER_SEC, MAX_PIXELS_PER_SEC)
}

impl NotationRenderer {
    pub fn new(score: Score, viewport_width: f32) -> Self {
        Self {
            score,
            pixels_per_sec: pixels_per_sec_for(viewport_width),
        }
    }

    /// Update horizontal scale to match the current viewport. Cheap; safe to
    /// call every frame.
    pub fn set_viewport_width(&mut self, viewport_w: f32) {
        self.pixels_per_sec = pixels_per_sec_for(viewport_w);
    }

    /// Notation-coordinate x position of an absolute time `t`.
    fn x_at(&self, t: Duration) -> f32 {
        t.saturating_sub(self.score.song_start).as_secs_f32() * self.pixels_per_sec
    }

    pub fn height_for_viewport(viewport_h: f32) -> f32 {
        (viewport_h * 0.25).clamp(180.0, 400.0)
    }

    /// X position (in notation coords) of the playhead at `time`. Linear in
    /// real time, so the playhead glides at constant pixels-per-second across
    /// the entire score.
    pub fn playhead_x(&self, time: Duration) -> f32 {
        self.x_at(time.clamp(self.score.song_start, self.score.song_end))
    }

    #[profiling::function]
    pub fn render(
        &self,
        quads: &mut QuadRenderer,
        target: &RenderTarget,
        current_time: Duration,
        color_schema: &[ColorSchemaV1],
    ) {
        let layout =
            compute_vertical_layout(&self.score, target.notation_height, target.top_offset);
        let scale = layout.scale();
        let pad = scale.nhw;
        let ctx = RenderCtx {
            scale,
            current_time,
            color_schema,
            scroll_x: target.scroll_x,
            vis_lo: target.scroll_x - pad,
            vis_hi: target.scroll_x + target.viewport_width + pad,
        };

        // Background.
        push_rect(
            quads,
            0.0,
            target.top_offset,
            target.viewport_width,
            target.notation_height,
            PAPER_WHITE,
        );

        // Staff lines.
        draw_staff_lines(
            quads,
            layout.treble_y,
            scale.ls,
            target.scroll_x,
            target.viewport_width,
        );
        draw_staff_lines(
            quads,
            layout.bass_y,
            scale.ls,
            target.scroll_x,
            target.viewport_width,
        );

        // Elements per staff.
        self.draw_staff(quads, &self.score.treble, layout.treble_y, &ctx);
        self.draw_staff(quads, &self.score.bass, layout.bass_y, &ctx);

        // Playhead — the scroll keeps `current_time` mapped to viewport center,
        // so the "now" indicator is just a static line at viewport_w / 2.
        push_rect(
            quads,
            target.viewport_width / 2.0 - PLAYHEAD_WIDTH / 2.0,
            target.top_offset,
            PLAYHEAD_WIDTH,
            target.notation_height,
            PLAYHEAD_COLOR,
        );
    }

    fn draw_staff(
        &self,
        quads: &mut QuadRenderer,
        staff: &Staff,
        sy: f32,
        ctx: &RenderCtx,
    ) {
        for el in &staff.elements {
            let nx = self.x_at(el.start_time());
            if nx < ctx.vis_lo || nx > ctx.vis_hi {
                continue;
            }
            let x = nx - ctx.scroll_x;
            match el {
                StaffElement::Chord(chord) => draw_chord(quads, chord, x, sy, ctx),
                StaffElement::Rest(rest) => draw_rest(quads, rest, x, sy, ctx.scale),
            }
        }
    }

}

// ── Free drawing helpers ───────────────────────────────────────────────

/// Convert a diatonic staff position to a y offset within a staff. Two
/// diatonic positions (one line + one space) make one line spacing, so the
/// scale factor is `ls / 2`. Higher staff_pos = smaller y on screen.
fn staff_pos_to_y(pos: StaffPos, ls: f32) -> f32 {
    ls * 4.0 - pos as f32 * ls / 2.0
}

fn draw_chord(quads: &mut QuadRenderer, chord: &Chord, x: f32, sy: f32, ctx: &RenderCtx) {
    let filled = chord.rhythmic_value != RhythmicValue::Whole
        && chord.rhythmic_value != RhythmicValue::Half;

    // Active color is shared across heads + stem; precompute once.
    let active_color = chord
        .heads
        .iter()
        .find(|h| ctx.current_time >= h.start_time && ctx.current_time < h.end_time)
        .map(|h| track_color(h.color_id, ctx.color_schema));

    for head in &chord.heads {
        let active = ctx.current_time >= head.start_time && ctx.current_time < head.end_time;
        let head_color = if active {
            track_color(head.color_id, ctx.color_schema)
        } else {
            INK_BLACK
        };
        draw_notehead(quads, head, x, sy, ctx.scale, head_color, filled);
        draw_ledger_lines(quads, head.staff_pos, x, sy, ctx.scale, head_color);
        if chord.dotted {
            // For noteheads on a line (even staff_pos), the dot is raised
            // into the space above; for space-noteheads the dot sits in
            // the same space.
            let on_line = head.staff_pos.rem_euclid(2) == 0;
            let y = sy + staff_pos_to_y(head.staff_pos, ctx.scale.ls)
                - if on_line { ctx.scale.ls / 2.0 } else { 0.0 };
            draw_dot(quads, x + ctx.scale.nhw * 0.7, y, ctx.scale, head_color);
        }
    }

    if chord.rhythmic_value != RhythmicValue::Whole {
        let stem_color = active_color.unwrap_or(INK_BLACK);
        draw_stem(quads, chord, x, sy, ctx.scale, stem_color);
    }
}

fn draw_stem(quads: &mut QuadRenderer, chord: &Chord, x: f32, sy: f32, scale: Scale, color: Color) {
    // Higher staff_pos = smaller y on screen, so `max_pos` gives the visually
    // top notehead and `min_pos` the visually bottom one.
    let (max_pos, min_pos) = chord
        .heads
        .iter()
        .fold((StaffPos::MIN, StaffPos::MAX), |(mx, mn), h| {
            (mx.max(h.staff_pos), mn.min(h.staff_pos))
        });
    let y_top = sy + staff_pos_to_y(max_pos, scale.ls);
    let y_bot = sy + staff_pos_to_y(min_pos, scale.ls);
    let height = scale.sh + (y_bot - y_top);

    let (stem_x, stem_y) = match chord.stem_direction {
        StemDirection::Up => (x + scale.nhw / 2.0 - scale.st / 2.0, y_top - scale.sh),
        StemDirection::Down => (x - scale.nhw / 2.0, y_top),
    };
    push_rect(quads, stem_x, stem_y, scale.st, height, color);
}

/// Push a flat-colored rectangle (no border radius).
fn push_rect(quads: &mut QuadRenderer, x: f32, y: f32, w: f32, h: f32, color: Color) {
    quads.push(QuadInstance {
        position: [x, y],
        size: [w, h],
        color: color.into_linear_rgba(),
        border_radius: [0.0; 4],
    });
}

/// Draw the five horizontal lines of a staff at vertical baseline `top_y`.
fn draw_staff_lines(quads: &mut QuadRenderer, top_y: f32, ls: f32, scroll_x: f32, viewport_w: f32) {
    let x = -scroll_x - STAFF_LINE_OVERDRAW;
    let w = viewport_w + scroll_x + STAFF_LINE_OVERDRAW * 2.0;
    for line in 0..5 {
        push_rect(quads, x, top_y + line as f32 * ls, w, 1.0, STAFF_GRAY);
    }
}

/// Draw a single notehead (filled for quarter+, donut-shaped for half/whole).
fn draw_notehead(
    quads: &mut QuadRenderer,
    head: &Notehead,
    x: f32,
    sy: f32,
    scale: Scale,
    color: Color,
    filled: bool,
) {
    let y = sy + staff_pos_to_y(head.staff_pos, scale.ls);
    let r = scale.nhh / 2.0;
    quads.push(QuadInstance {
        position: [x - scale.nhw / 2.0, y - scale.nhh / 2.0],
        size: [scale.nhw, scale.nhh],
        color: color.into_linear_rgba(),
        border_radius: [r; 4],
    });
    if !filled {
        let b = scale.st;
        let inner_r = (scale.nhh - b * 2.0) / 2.0;
        quads.push(QuadInstance {
            position: [x - scale.nhw / 2.0 + b, y - scale.nhh / 2.0 + b],
            size: [scale.nhw - b * 2.0, scale.nhh - b * 2.0],
            color: PAPER_WHITE.into_linear_rgba(),
            border_radius: [inner_r; 4],
        });
    }
}

/// Draw ledger lines for a notehead that sits above or below the staff.
fn draw_ledger_lines(
    quads: &mut QuadRenderer,
    pos: StaffPos,
    x: f32,
    sy: f32,
    scale: Scale,
    color: Color,
) {
    let lw = scale.nhw + 6.0;
    let mut draw_line = |lp: StaffPos| {
        let ly = sy + staff_pos_to_y(lp, scale.ls);
        push_rect(quads, x - lw / 2.0, ly - scale.st / 2.0, lw, scale.st, color);
    };
    if pos < 0 {
        // -2, -4, -6, ... down to pos (each ledger line is 2 diatonic steps).
        let mut lp: StaffPos = -2;
        while lp >= pos {
            draw_line(lp);
            lp -= 2;
        }
    } else if pos > TOP_LINE_POS {
        let mut lp: StaffPos = TOP_LINE_POS + 2;
        while lp <= pos {
            draw_line(lp);
            lp += 2;
        }
    }
}

/// Draw a small filled circle (used for augmentation dots).
fn draw_dot(quads: &mut QuadRenderer, x: f32, y: f32, scale: Scale, color: Color) {
    let dot = scale.nhh * 0.45;
    quads.push(QuadInstance {
        position: [x, y - dot / 2.0],
        size: [dot, dot],
        color: color.into_linear_rgba(),
        border_radius: [dot / 2.0; 4],
    });
}

/// Draw a rest glyph (and its augmentation dot if any). Whole rests hang
/// from the 4th line (the line above the middle line); half rests sit on
/// the middle line; shorter rests are centered on the middle line.
fn draw_rest(quads: &mut QuadRenderer, rest: &Rest, x: f32, sy: f32, scale: Scale) {
    let line_pos: StaffPos = match rest.rhythmic_value {
        RhythmicValue::Whole => MIDDLE_LINE_POS + 2,
        _ => MIDDLE_LINE_POS,
    };
    let line_y = sy + staff_pos_to_y(line_pos, scale.ls);
    let h = scale.nhh * 0.7;
    let top_y = match rest.rhythmic_value {
        RhythmicValue::Whole => line_y,        // hang below the 4th line
        RhythmicValue::Half => line_y - h,     // sit above the middle line
        _ => line_y - h / 2.0,                 // centered on the middle line
    };
    quads.push(QuadInstance {
        position: [x - scale.nhw * 0.4, top_y],
        size: [scale.nhw * 0.8, h],
        color: INK_BLACK.into_linear_rgba(),
        border_radius: [1.0; 4],
    });
    if rest.dotted {
        // Dot in the space above the rest's anchoring line.
        draw_dot(
            quads,
            x + scale.nhw * 0.5,
            line_y - scale.ls / 4.0,
            scale,
            INK_BLACK,
        );
    }
}

// ── Vertical layout ────────────────────────────────────────────────────

/// Y baselines and pixel sizes for one frame's worth of rendering.
struct VerticalLayout {
    treble_y: f32,
    bass_y: f32,
    /// Uniform scale factor applied to all `BASE_*` constants.
    s: f32,
}

impl VerticalLayout {
    fn scale(&self) -> Scale {
        Scale {
            ls: BASE_LINE_SPACING * self.s,
            nhw: BASE_NOTE_HEAD_WIDTH * self.s,
            nhh: BASE_NOTE_HEAD_HEIGHT * self.s,
            sh: BASE_STEM_HEIGHT * self.s,
            st: BASE_STEM_THICKNESS * self.s,
        }
    }
}

/// Padding (in `BASE_LINE_SPACING` units) needed above the staff: one line
/// of breathing room plus one line spacing per pair of diatonic positions
/// above the top staff line.
fn pad_above(range: StaffRange) -> f32 {
    let lines_above = (range.1 - TOP_LINE_POS).max(0) as f32 / 2.0;
    (lines_above + 1.0) * BASE_LINE_SPACING
}

/// Padding needed below the staff: one line of breathing room plus one line
/// spacing per pair of diatonic positions below the bottom staff line.
fn pad_below(range: StaffRange) -> f32 {
    let lines_below = (-range.0).max(0) as f32 / 2.0;
    (lines_below + 1.0) * BASE_LINE_SPACING
}

/// Compute the vertical layout for the grand staff: how to allocate `notation_h`
/// pixels across treble padding, treble staff, gap, bass staff, bass padding,
/// scaled to fit the actual ledger-line range of each staff.
fn compute_vertical_layout(score: &Score, notation_h: f32, top_offset: f32) -> VerticalLayout {
    let staff_h = BASE_LINE_SPACING * 4.0;
    let gap = BASE_LINE_SPACING * 3.0;
    let raw_tt = pad_above(score.treble.range);
    let raw_tb = pad_below(score.treble.range);
    let raw_bt = pad_above(score.bass.range);
    let raw_bb = pad_below(score.bass.range);
    let raw_total = (raw_tt + staff_h + raw_tb + gap + raw_bt + staff_h + raw_bb).max(1.0);

    let s = notation_h / raw_total;
    let treble_y = top_offset + raw_tt * s;
    let bass_y = top_offset + (raw_tt + staff_h + raw_tb + gap + raw_bt) * s;
    VerticalLayout {
        treble_y,
        bass_y,
        s,
    }
}
