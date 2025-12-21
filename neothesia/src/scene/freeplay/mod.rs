use std::time::Duration;

use midi_file::midly::MidiMessage;
use neothesia_core::render::{GuidelineRenderer, QuadRenderer, TextRenderer};
use winit::{
    event::{ElementState, KeyEvent, MouseButton, WindowEvent},
    keyboard::{Key, NamedKey},
};

use crate::{
    NeothesiaEvent,
    context::Context,
    scene::{Scene, playing_scene::Keyboard},
    song::Song,
};

pub struct FreeplayScene {
    keyboard: Keyboard,
    guidelines: GuidelineRenderer,

    text_renderer: TextRenderer,
    quad_renderer_bg: QuadRenderer,
    quad_renderer_fg: QuadRenderer,

    // TODO: This does not make sens, but get's us going without refactoring
    song: Option<Song>,
}

impl FreeplayScene {
    pub fn new(ctx: &mut Context, song: Option<Song>) -> Self {
        let mut keyboard = Keyboard::new(ctx, Default::default());
        keyboard.set_pressed_by_user_colors(ctx.config.color_schema()[0].clone());

        let keyboard_layout = keyboard.layout();

        let guidelines = GuidelineRenderer::new(
            keyboard_layout.clone(),
            *keyboard.pos(),
            ctx.config.vertical_guidelines(),
            false,
            Default::default(),
        );

        let text_renderer = ctx.text_renderer_factory.new_renderer();

        let quad_renderer_bg = ctx.quad_renderer_facotry.new_renderer();
        let quad_renderer_fg = ctx.quad_renderer_facotry.new_renderer();

        Self {
            keyboard,
            guidelines,
            text_renderer,
            quad_renderer_bg,
            quad_renderer_fg,
            song,
        }
    }

    fn resize(&mut self, ctx: &mut Context) {
        self.keyboard.resize(ctx);
        self.guidelines.set_layout(self.keyboard.layout().clone());
        self.guidelines.set_pos(*self.keyboard.pos());
    }
}

impl Scene for FreeplayScene {
    fn update(&mut self, ctx: &mut Context, _delta: Duration) {
        self.quad_renderer_bg.clear();
        self.quad_renderer_fg.clear();

        let time = 0.0;

        self.guidelines.update(
            &mut self.quad_renderer_bg,
            ctx.config.animation_speed(),
            ctx.window_state.scale_factor as f32,
            time,
            ctx.window_state.logical_size,
        );
        self.keyboard
            .update(&mut self.quad_renderer_fg, &mut self.text_renderer);

        self.quad_renderer_bg.prepare();
        self.quad_renderer_fg.prepare();
        self.text_renderer.update(
            ctx.window_state.physical_size,
            ctx.window_state.scale_factor as f32,
        );
    }

    fn render<'pass>(&'pass mut self, rpass: &mut wgpu_jumpstart::RenderPass<'pass>) {
        self.quad_renderer_bg.render(rpass);
        self.quad_renderer_fg.render(rpass);
        self.text_renderer.render(rpass);
    }

    fn window_event(&mut self, ctx: &mut Context, event: &WindowEvent) {
        if let WindowEvent::Resized(_) | WindowEvent::ScaleFactorChanged { .. } = event {
            self.resize(ctx)
        }

        handle_back_button(ctx, event, self.song.as_ref());
        super::handle_pc_keyboard_to_midi_event(ctx, event);
    }

    fn midi_event(&mut self, ctx: &mut Context, _channel: u8, message: &MidiMessage) {
        self.keyboard.user_midi_event(message);
        ctx.output_manager
            .connection()
            .midi_event(0.into(), *message);
    }
}

fn handle_back_button(ctx: &Context, event: &WindowEvent, song: Option<&Song>) {
    let mut is_back_event = matches!(
        event,
        WindowEvent::KeyboardInput {
            event: KeyEvent {
                state: ElementState::Released,
                logical_key: Key::Named(NamedKey::Escape),
                ..
            },
            ..
        }
    );

    is_back_event |= matches!(
        event,
        WindowEvent::MouseInput {
            state: ElementState::Pressed,
            button: MouseButton::Back,
            ..
        }
    );

    if is_back_event {
        ctx.proxy
            .send_event(NeothesiaEvent::MainMenu(song.cloned()))
            .ok();
    }
}
