use neothesia_core::render::NoteLabels;

use super::Keyboard;
use crate::{context::Context, render::WaterfallRenderer, song::Song};

pub struct PlaybackVisuals {
    waterfall: WaterfallRenderer,
    note_labels: Option<NoteLabels>,
}

impl PlaybackVisuals {
    pub fn new(ctx: &Context, song: &Song, keyboard: &Keyboard) -> Self {
        let hidden_tracks: Vec<usize> = song
            .config
            .tracks
            .iter()
            .filter(|track| !track.visible)
            .map(|track| track.track_id)
            .collect();

        let waterfall = WaterfallRenderer::new(
            &ctx.gpu,
            &song.file.tracks,
            &hidden_tracks,
            &ctx.config,
            &ctx.transform,
            keyboard.layout().clone(),
        );

        let note_labels = ctx.config.note_labels().then_some(NoteLabels::new(
            *keyboard.pos(),
            waterfall.notes(),
            ctx.text_renderer_factory.new_renderer(),
        ));

        Self {
            waterfall,
            note_labels,
        }
    }

    pub fn resize(&mut self, ctx: &Context, keyboard: &Keyboard) {
        if let Some(note_labels) = self.note_labels.as_mut() {
            note_labels.set_pos(*keyboard.pos());
        }

        self.waterfall.resize(&ctx.config, keyboard.layout().clone());
    }

    pub fn update_waterfall(&mut self, time: f32) {
        self.waterfall.update(time);
    }

    pub fn update_note_labels(&mut self, ctx: &Context, keyboard: &Keyboard, time: f32) {
        if let Some(note_labels) = self.note_labels.as_mut() {
            note_labels.update(
                ctx.window_state.physical_size,
                ctx.window_state.scale_factor as f32,
                keyboard.renderer(),
                ctx.config.animation_speed(),
                time,
            );
        }
    }

    pub fn set_animation_speed(&mut self, ctx: &Context) {
        self.waterfall
            .pipeline()
            .set_speed(&ctx.gpu.queue, ctx.config.animation_speed());
    }

    pub fn render<'pass>(&'pass mut self, rpass: &mut wgpu_jumpstart::RenderPass<'pass>) {
        self.waterfall.render(rpass);
        if let Some(note_labels) = self.note_labels.as_mut() {
            note_labels.render(rpass);
        }
    }
}