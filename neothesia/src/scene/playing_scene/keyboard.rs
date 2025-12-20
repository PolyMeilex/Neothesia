use midi_file::midly::MidiMessage;
use neothesia_core::{
    config::ColorSchemaV1,
    piano_layout,
    render::{KeyboardKeyState, QuadRenderer, TextRenderer},
    utils::Point,
};
use piano_layout::KeyboardRange;

use crate::{config::Config, context::Context, render::KeyboardRenderer, song::SongConfig};

pub struct Keyboard {
    renderer: KeyboardRenderer,
    song_config: SongConfig,
    pressed_by_user_colors: ColorSchemaV1,
}

fn get_layout(
    width: f32,
    height: f32,
    range: piano_layout::KeyboardRange,
) -> piano_layout::KeyboardLayout {
    let white_count = range.white_count();
    let neutral_width = width / white_count as f32;
    let neutral_height = height * 0.2;

    piano_layout::KeyboardLayout::from_range(
        piano_layout::Sizing::new(neutral_width, neutral_height),
        range,
    )
}

impl Keyboard {
    pub fn new(ctx: &Context, song_config: SongConfig) -> Self {
        let layout = get_layout(
            ctx.window_state.logical_size.width,
            ctx.window_state.logical_size.height,
            piano_layout::KeyboardRange::new(ctx.config.piano_range()),
        );

        let mut renderer = KeyboardRenderer::new(layout);
        renderer.position_on_bottom_of_parent(ctx.window_state.logical_size.height);

        let v = (255.0 * 0.3) as u8;
        let dark = (v, v, v);

        let v = (255.0 * 0.5) as u8;
        let base = (v, v, v);

        Self {
            renderer,
            song_config,
            pressed_by_user_colors: ColorSchemaV1 { base, dark },
        }
    }

    pub fn set_pressed_by_user_colors(&mut self, colors: ColorSchemaV1) {
        self.pressed_by_user_colors = colors;
    }

    pub fn renderer(&self) -> &KeyboardRenderer {
        &self.renderer
    }

    pub fn key_states(&self) -> &[KeyboardKeyState] {
        self.renderer.key_states()
    }

    pub fn layout(&self) -> &piano_layout::KeyboardLayout {
        self.renderer.layout()
    }

    fn set_layout(&mut self, layout: piano_layout::KeyboardLayout) {
        self.renderer.set_layout(layout)
    }

    pub fn range(&self) -> &KeyboardRange {
        &self.layout().range
    }

    pub fn pos(&self) -> &Point<f32> {
        self.renderer.pos()
    }

    fn position_on_bottom_of_parent(&mut self, parent_height: f32) {
        self.renderer.position_on_bottom_of_parent(parent_height)
    }

    #[profiling::function]
    pub fn resize(&mut self, ctx: &Context) {
        let keyboard_layout = get_layout(
            ctx.window_state.logical_size.width,
            ctx.window_state.logical_size.height,
            self.renderer.layout().range.clone(),
        );

        self.set_layout(keyboard_layout.clone());
        self.position_on_bottom_of_parent(ctx.window_state.logical_size.height);
    }

    pub fn update(&mut self, quads: &mut QuadRenderer, brush: &mut TextRenderer) {
        self.renderer.update(quads, brush)
    }

    pub fn reset_notes(&mut self) {
        self.renderer.reset_notes()
    }

    pub fn user_midi_event(&mut self, message: &MidiMessage) {
        let range_start = self.range().start() as usize;

        let (is_on, key) = match message {
            MidiMessage::NoteOn { key, .. } => (true, key.as_int()),
            MidiMessage::NoteOff { key, .. } => (false, key.as_int()),
            _ => return,
        };

        if self.range().contains(key) {
            let id = key as usize - range_start;
            let key = &mut self.renderer.key_states_mut()[id];

            key.set_pressed_by_user(is_on, &self.pressed_by_user_colors);
            self.renderer.invalidate_cache();
        }
    }

    pub fn file_midi_events(&mut self, config: &Config, events: &[&midi_file::MidiEvent]) {
        let range_start = self.range().start() as usize;

        for e in events {
            let track = &self.song_config.tracks[e.track_id];
            if !track.visible {
                continue;
            }

            let (is_on, key) = match e.message {
                MidiMessage::NoteOn { key, .. } => (true, key.as_int()),
                MidiMessage::NoteOff { key, .. } => (false, key.as_int()),
                _ => continue,
            };

            if self.range().contains(key) && e.channel != 9 {
                let id = key as usize - range_start;
                let key = &mut self.renderer.key_states_mut()[id];

                if is_on {
                    let color =
                        &config.color_schema()[e.track_color_id % config.color_schema().len()];
                    key.pressed_by_file_on(color);
                } else {
                    key.pressed_by_file_off();
                }

                self.renderer.invalidate_cache();
            }
        }
    }
}
