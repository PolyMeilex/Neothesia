use midi_file::midly::MidiMessage;
use neothesia_core::render::{GuidelineRenderer, QuadPipeline};
use std::time::Duration;
use wgpu_jumpstart::{TransformUniform, Uniform};
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
use crate::menu_scene::Step;

const EVENT_CAPTURED: bool = true;
const EVENT_IGNORED: bool = false;

pub struct PlayingScene {
    keyboard: Keyboard,
    waterfall: WaterfallRenderer,
    guidelines: GuidelineRenderer,

    player: MidiPlayer,
    rewind_controller: RewindController,
    bg_quad_pipeline: QuadPipeline,
    fg_quad_pipeline: QuadPipeline,
    toast_manager: ToastManager,

    top_bar: TopBar,
}

impl PlayingScene {
    pub fn new(ctx: &Context, song: Song) -> Self {
        let keyboard = Keyboard::new(ctx, song.config.clone());

        let keyboard_layout = keyboard.layout();

        let guidelines = GuidelineRenderer::new(
            keyboard_layout.clone(),
            *keyboard.pos(),
            ctx.config.vertical_guidelines,
            ctx.config.horizontal_guidelines,
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

        let mut player = MidiPlayer::new(
            ctx.output_manager.connection().clone(),
            song,
            keyboard_layout.range.clone(),
        );
        let weak_ctx = ctx.proxy.clone();
        player.on_finish(move || {
            weak_ctx
                .send_event(NeothesiaEvent::MainMenu { page: Step::Stats })
                .ok();
        });
        waterfall.update(&ctx.gpu.queue, player.time_without_lead_in());

        Self {
            keyboard,
            guidelines,

            waterfall,
            player,
            rewind_controller: RewindController::new(),
            bg_quad_pipeline: QuadPipeline::new(&ctx.gpu, &ctx.transform),
            fg_quad_pipeline: QuadPipeline::new(&ctx.gpu, &ctx.transform),
            toast_manager: ToastManager::default(),
            top_bar: TopBar::new(),
        }
    }

    fn update_midi_player(&mut self, ctx: &Context, delta: Duration) -> f32 {
        if self.top_bar.loop_active && self.player.time() > self.top_bar.loop_end {
            self.player.set_time(self.top_bar.loop_start);
            self.keyboard.reset_notes();
        }

        if self.player.play_along().are_required_keys_pressed() {
            let delta = (delta / 10) * (ctx.config.speed_multiplier * 10.0) as u32;
            let midi_events = self.player.update(delta);
            self.keyboard.file_midi_events(&ctx.config, &midi_events);
        }

        self.player.time_without_lead_in() + ctx.config.playback_offset
    }

    fn resize(&mut self, ctx: &mut Context) {
        self.keyboard.resize(ctx);

        self.guidelines.set_layout(self.keyboard.layout().clone());
        self.guidelines.set_pos(*self.keyboard.pos());

        self.waterfall
            .resize(&ctx.gpu.queue, &ctx.config, self.keyboard.layout().clone());
    }
}

impl Scene for PlayingScene {
    fn update(&mut self, ctx: &mut Context, delta: Duration) {
        self.bg_quad_pipeline.clear();
        self.fg_quad_pipeline.clear();

        self.rewind_controller.update(&mut self.player, ctx);
        self.toast_manager.update(&mut ctx.text_renderer);

        let time = self.update_midi_player(ctx, delta);
        self.waterfall.update(&ctx.gpu.queue, time);
        self.guidelines
            .update(&mut self.bg_quad_pipeline, ctx.config.animation_speed, time);
        self.keyboard
            .update(&mut self.fg_quad_pipeline, &mut ctx.text_renderer);

        TopBar::update(self, &ctx.window_state, &mut ctx.text_renderer);

        self.bg_quad_pipeline.prepare(&ctx.gpu.queue);
        self.fg_quad_pipeline.prepare(&ctx.gpu.queue);
    }

    fn render<'pass>(
        &'pass mut self,
        transform: &'pass Uniform<TransformUniform>,
        rpass: &mut wgpu::RenderPass<'pass>,
    ) {
        self.bg_quad_pipeline.render(transform, rpass);
        self.waterfall.render(transform, rpass);
        self.fg_quad_pipeline.render(transform, rpass);
    }

    fn window_event(&mut self, ctx: &mut Context, event: &WindowEvent) {
        if TopBar::handle_window_event(self, ctx, event) {
            return;
        }

        self.rewind_controller
            .handle_window_event(ctx, event, &mut self.player);

        if self.rewind_controller.is_rewinding() {
            self.keyboard.reset_notes();
        }

        handle_back_button(ctx, event);
        handle_pause_button(&mut self.player, event);
        handle_settings_input(ctx, &mut self.toast_manager, &mut self.waterfall, event);

        if let WindowEvent::Resized(_) | WindowEvent::ScaleFactorChanged { .. } = event {
            self.resize(ctx)
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

fn handle_back_button(ctx: &Context, event: &WindowEvent) {
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
            .send_event(NeothesiaEvent::MainMenu { page: Step::Main })
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
                ctx.config.speed_multiplier += amount;
            } else {
                ctx.config.speed_multiplier -= amount;
                ctx.config.speed_multiplier = ctx.config.speed_multiplier.max(0.0);
            }

            toast_manager.speed_toast(ctx.config.speed_multiplier);
        }

        Key::Named(key @ (NamedKey::PageUp | NamedKey::PageDown)) => {
            let amount = if ctx.window_state.modifiers_state.shift_key() {
                500.0
            } else {
                100.0
            };

            if key == NamedKey::PageUp {
                ctx.config.animation_speed += amount;
                // 0.0 is invalid speed, let's skip it and jump to positive
                if ctx.config.animation_speed == 0.0 {
                    ctx.config.animation_speed += amount;
                }
            } else {
                ctx.config.animation_speed -= amount;
                // 0.0 is invalid speed, let's skip it and jump to negative
                if ctx.config.animation_speed == 0.0 {
                    ctx.config.animation_speed -= amount;
                }
            }

            waterfall
                .pipeline()
                .set_speed(&ctx.gpu.queue, ctx.config.animation_speed);
            toast_manager.animation_speed_toast(ctx.config.animation_speed);
        }

        Key::Character(ref ch) if matches!(ch.as_str(), "_" | "-" | "+" | "=") => {
            let amount = if ctx.window_state.modifiers_state.shift_key() {
                0.1
            } else {
                0.01
            };

            if matches!(ch.as_str(), "-" | "_") {
                ctx.config.playback_offset -= amount;
            } else {
                ctx.config.playback_offset += amount;
            }

            toast_manager.offset_toast(ctx.config.playback_offset);
        }

        _ => {}
    }
}
