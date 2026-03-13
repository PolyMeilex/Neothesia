mod ui;

use crate::{
    context::Context, scene::Scene, song::Song, utils::window::WinitEvent, NeothesiaEvent,
};
use neothesia_core::render::{QuadRenderer, TextRenderer};
use std::time::Duration;
use winit::event::WindowEvent;

use super::NuonRenderer;
use crate::scene::playing_scene::midi_player::ScoreData;

pub struct ScoreScene {
    quad_renderer: QuadRenderer,
    text_renderer: TextRenderer,
    nuon_renderer: NuonRenderer,
    nuon: nuon::Ui,

    song: Song,
    score_data: ScoreData,
}

impl ScoreScene {
    pub fn new(ctx: &mut Context, song: Song, score_data: ScoreData) -> Self {
        let quad_renderer = ctx.quad_renderer_facotry.new_renderer();
        let text_renderer = ctx.text_renderer_factory.new_renderer();
        let nuon_renderer = NuonRenderer::new(ctx);

        Self {
            quad_renderer,
            text_renderer,
            nuon_renderer,
            nuon: nuon::Ui::new(),
            song,
            score_data,
        }
    }
}

impl Scene for ScoreScene {
    fn update(&mut self, ctx: &mut Context, _delta: Duration) {
        self.quad_renderer.clear();

        ui::render_score_ui(self, ctx);

        super::render_nuon(&mut self.nuon, &mut self.nuon_renderer, ctx);

        self.quad_renderer.prepare();

        self.text_renderer.update(
            ctx.window_state.physical_size,
            ctx.window_state.scale_factor as f32,
        );
    }

    fn render<'pass>(&'pass mut self, rpass: &mut wgpu_jumpstart::RenderPass<'pass>) {
        self.quad_renderer.render(rpass);
        self.text_renderer.render(rpass);
        self.nuon_renderer.render(rpass);
    }

    fn window_event(&mut self, ctx: &mut Context, event: &WindowEvent) {
        if event.key_released(winit::keyboard::Key::Named(
            winit::keyboard::NamedKey::Escape,
        )) {
            ctx.proxy
                .send_event(NeothesiaEvent::MainMenu(Some(self.song.clone())))
                .ok();
        }

        super::handle_nuon_window_event(&mut self.nuon, event, ctx);
    }
}
