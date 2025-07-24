use midi_file::midly::MidiMessage;
use neothesia_core::render::{
    GlowInstance, GlowPipeline, GuidelineRenderer, NoteLabels, QuadRenderer, TextRenderer,
};
use std::time::Duration;
use winit::{
    event::{ElementState, KeyEvent, MouseButton, WindowEvent},
    keyboard::{Key, NamedKey},
};

use self::top_bar::TopBar;

use super::Scene;
use crate::{context::Context, render::WaterfallRenderer, song::Song, NeothesiaEvent};

mod keyboard;
use keyboard::Keyboard;

mod midi_player;
use midi_player::MidiPlayer;

mod rewind_controller;
use rewind_controller::RewindController;

mod toast_manager;
use toast_manager::ToastManager;

mod animation;
mod top_bar;

const LAYER_BG: usize = 0;
const LAYER_FG: usize = 1;

struct GlowState {
    time: f32,
}

struct Glow {
    pipeline: GlowPipeline,
    states: Vec<GlowState>,
}

pub struct PlayingScene {
    keyboard: Keyboard,
    waterfall: WaterfallRenderer,
    guidelines: GuidelineRenderer,
    text_renderer: TextRenderer,

    note_labels: Option<NoteLabels>,

    player: MidiPlayer,
    rewind_controller: RewindController,
    quad_pipeline: QuadRenderer,
    glow: Option<Glow>,
    toast_manager: ToastManager,

    nuon: nuon::Ui,

    top_bar: TopBar,
}

impl PlayingScene {
    pub fn new(ctx: &mut Context, song: Song) -> Self {
        let keyboard = Keyboard::new(ctx, song.config.clone());

        let keyboard_layout = keyboard.layout();

        let guidelines = GuidelineRenderer::new(
            keyboard_layout.clone(),
            *keyboard.pos(),
            ctx.config.vertical_guidelines(),
            ctx.config.horizontal_guidelines(),
            song.file.measures.clone(),
        );

        let hidden_tracks: Vec<usize> = song
            .config
            .tracks
            .iter()
            .filter(|t| !t.visible)
            .map(|t| t.track_id)
            .collect();

        let mut waterfall = WaterfallRenderer::new(
            &ctx.gpu,
            &song.file.tracks,
            &hidden_tracks,
            &ctx.config,
            &ctx.transform,
            keyboard_layout.clone(),
        );

        let mut text_renderer = TextRenderer::new(&ctx.gpu);

        let note_labels = ctx.config.note_labels().then_some(NoteLabels::new(
            *keyboard.pos(),
            waterfall.notes(),
            text_renderer.new_renderer(),
        ));

        let player = MidiPlayer::new(
            ctx.output_manager.connection().clone(),
            song,
            keyboard_layout.range.clone(),
            ctx.config.separate_channels(),
        );
        waterfall.update(player.time_without_lead_in());

        let mut quad_pipeline = QuadRenderer::new(&ctx.gpu, &ctx.transform);
        quad_pipeline.init_layer(50); // BG
        quad_pipeline.init_layer(150); // FG

        let glow_states: Vec<GlowState> = keyboard
            .layout()
            .range
            .iter()
            .map(|_| GlowState { time: 0.0 })
            .collect();

        Self {
            keyboard,
            guidelines,
            note_labels,
            text_renderer,

            waterfall,
            player,
            rewind_controller: RewindController::new(),
            quad_pipeline,
            glow: ctx.config.glow().then_some(Glow {
                pipeline: GlowPipeline::new(&ctx.gpu, &ctx.transform),
                states: glow_states,
            }),
            toast_manager: ToastManager::default(),

            nuon: nuon::Ui::new(),

            top_bar: TopBar::new(),
        }
    }

    fn update_glow(&mut self, delta: Duration) {
        let Some(glow) = &mut self.glow else {
            return;
        };

        glow.pipeline.clear();

        let key_states = self.keyboard.key_states();
        for key in self.keyboard.layout().keys.iter() {
            let glow_state = &mut glow.states[key.id()];
            let glow_w = 150.0 + glow_state.time.sin() * 10.0;
            let glow_h = 150.0 + glow_state.time.sin() * 10.0;

            let y = self.keyboard.pos().y;
            if let Some(color) = key_states[key.id()].pressed_by_file() {
                glow_state.time += delta.as_secs_f32() * 5.0;
                let mut color = color.into_linear_rgba();
                let v = 0.2 * glow_state.time.cos().abs();
                let v = v.min(1.0);
                color[0] += v;
                color[1] += v;
                color[2] += v;
                color[3] = 0.2;
                glow.pipeline.instances().push(GlowInstance {
                    position: [key.x() - glow_w / 2.0 + key.width() / 2.0, y - glow_w / 2.0],
                    size: [glow_w, glow_h],
                    color,
                });
            }
        }
    }

    #[profiling::function]
    fn update_midi_player(&mut self, ctx: &Context, delta: Duration) -> f32 {
        if self.top_bar.is_looper_active() && self.player.time() > self.top_bar.loop_end_timestamp()
        {
            self.player.set_time(self.top_bar.loop_start_timestamp());
            self.keyboard.reset_notes();
        }

        if self.player.play_along().are_required_keys_pressed() {
            let delta = (delta / 10) * (ctx.config.speed_multiplier() * 10.0) as u32;
            let midi_events = self.player.update(delta);
            self.keyboard.file_midi_events(&ctx.config, &midi_events);
        }

        self.player.time_without_lead_in() + ctx.config.animation_offset()
    }

    #[profiling::function]
    fn resize(&mut self, ctx: &mut Context) {
        self.keyboard.resize(ctx);

        self.guidelines.set_layout(self.keyboard.layout().clone());
        self.guidelines.set_pos(*self.keyboard.pos());
        if let Some(note_labels) = self.note_labels.as_mut() {
            note_labels.set_pos(*self.keyboard.pos());
        }

        self.waterfall
            .resize(&ctx.config, self.keyboard.layout().clone());
    }
}

impl Scene for PlayingScene {
    #[profiling::function]
    fn update(&mut self, ctx: &mut Context, delta: Duration) {
        self.text_renderer.end_frame();
        self.quad_pipeline.clear();

        self.rewind_controller.update(&mut self.player, ctx, delta);
        self.toast_manager.update(&mut self.text_renderer);

        let time = self.update_midi_player(ctx, delta);
        self.waterfall.update(time);
        self.guidelines.update(
            &mut self.quad_pipeline,
            LAYER_BG,
            ctx.config.animation_speed(),
            time,
        );
        self.keyboard
            .update(&mut self.quad_pipeline, LAYER_FG, &mut self.text_renderer);
        if let Some(note_labels) = self.note_labels.as_mut() {
            note_labels.update(
                ctx.window_state.logical_size.into(),
                self.keyboard.renderer(),
                ctx.config.animation_speed(),
                time,
            );
        }

        self.update_glow(delta);

        TopBar::update(self, ctx);

        super::render_nuon(
            &mut self.nuon,
            &mut self.quad_pipeline,
            LAYER_FG,
            &mut self.text_renderer,
            &mut ctx.iced_manager.renderer,
        );

        self.quad_pipeline.prepare();
        if let Some(glow) = &mut self.glow {
            glow.pipeline.prepare();
        }

        #[cfg(debug_assertions)]
        self.text_renderer.queue_fps(
            ctx.fps_ticker.avg(),
            self.top_bar
                .topbar_expand_animation
                .animate_bool(5.0, 80.0, ctx.frame_timestamp),
        );
        self.text_renderer
            .update(ctx.window_state.logical_size.into());

        if self.player.is_finished() && !self.player.is_paused() {
            ctx.proxy
                .send_event(NeothesiaEvent::MainMenu(Some(self.player.song().clone())))
                .ok();
        }
    }

    #[profiling::function]
    fn render<'pass>(&'pass mut self, rpass: &mut wgpu::RenderPass<'pass>) {
        self.quad_pipeline.render(LAYER_BG, rpass);
        self.waterfall.render(rpass);
        if let Some(note_labels) = self.note_labels.as_mut() {
            note_labels.render(rpass);
        }
        self.quad_pipeline.render(LAYER_FG, rpass);
        if let Some(glow) = &self.glow {
            glow.pipeline.render(rpass);
        }
        self.text_renderer.render(rpass);
    }

    fn window_event(&mut self, ctx: &mut Context, event: &WindowEvent) {
        self.rewind_controller
            .handle_window_event(ctx, event, &mut self.player);

        if self.rewind_controller.is_rewinding() {
            self.keyboard.reset_notes();
        }

        handle_back_button(ctx, self.player.song(), event);
        handle_pause_button(&mut self.player, event);
        handle_settings_input(ctx, &mut self.toast_manager, &mut self.waterfall, event);

        if let WindowEvent::Resized(_) | WindowEvent::ScaleFactorChanged { .. } = event {
            self.resize(ctx)
        }

        if let WindowEvent::CursorMoved { .. } = event {
            self.nuon.mouse_move(
                ctx.window_state.cursor_logical_position.x,
                ctx.window_state.cursor_logical_position.y,
            );
        }

        if let WindowEvent::MouseInput {
            state,
            button: MouseButton::Left,
            ..
        } = event
        {
            match state {
                ElementState::Pressed => self.nuon.mouse_down(),
                ElementState::Released => self.nuon.mouse_up(),
            }
        }
    }

    fn midi_event(&mut self, _ctx: &mut Context, _channel: u8, message: &MidiMessage) {
        self.player
            .play_along_mut()
            .midi_event(midi_player::MidiEventSource::User, message);
        self.keyboard.user_midi_event(message);
    }
}

fn handle_pause_button(player: &mut MidiPlayer, event: &WindowEvent) {
    match event {
        WindowEvent::KeyboardInput {
            event:
                KeyEvent {
                    state: ElementState::Released,
                    logical_key: Key::Named(NamedKey::Space),
                    ..
                },
            ..
        } => {
            player.pause_resume();
        }
        _ => {}
    }
}

fn handle_back_button(ctx: &Context, song: &Song, event: &WindowEvent) {
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
            .send_event(NeothesiaEvent::MainMenu(Some(song.clone())))
            .ok();
    }
}

fn handle_settings_input(
    ctx: &mut Context,
    toast_manager: &mut ToastManager,
    waterfall: &mut WaterfallRenderer,
    event: &WindowEvent,
) {
    let WindowEvent::KeyboardInput { event, .. } = event else {
        return;
    };

    if event.state != ElementState::Released {
        return;
    }

    match event.logical_key {
        Key::Named(key @ (NamedKey::ArrowUp | NamedKey::ArrowDown)) => {
            let amount = if ctx.window_state.modifiers_state.shift_key() {
                0.5
            } else {
                0.1
            };

            if key == NamedKey::ArrowUp {
                ctx.config
                    .set_speed_multiplier(ctx.config.speed_multiplier() + amount);
            } else {
                ctx.config
                    .set_speed_multiplier(ctx.config.speed_multiplier() - amount);
            }

            toast_manager.speed_toast(ctx.config.speed_multiplier());
        }

        Key::Named(key @ (NamedKey::PageUp | NamedKey::PageDown)) => {
            let amount = if ctx.window_state.modifiers_state.shift_key() {
                500.0
            } else {
                100.0
            };

            if key == NamedKey::PageUp {
                ctx.config
                    .set_animation_speed(ctx.config.animation_speed() + amount);
            } else {
                ctx.config
                    .set_animation_speed(ctx.config.animation_speed() - amount);
            }

            waterfall
                .pipeline()
                .set_speed(&ctx.gpu.queue, ctx.config.animation_speed());
            toast_manager.animation_speed_toast(ctx.config.animation_speed());
        }

        Key::Character(ref ch) if matches!(ch.as_str(), "_" | "-" | "+" | "=") => {
            let amount = if ctx.window_state.modifiers_state.shift_key() {
                0.1
            } else {
                0.01
            };

            if matches!(ch.as_str(), "-" | "_") {
                ctx.config
                    .set_animation_offset(ctx.config.animation_offset() - amount);
            } else {
                ctx.config
                    .set_animation_offset(ctx.config.animation_offset() + amount);
            }

            toast_manager.offset_toast(ctx.config.animation_offset());
        }

        _ => {}
    }
}
