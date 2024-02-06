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
use crate::{render::WaterfallRenderer, song::Song, target::Target, NeothesiaEvent};

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

pub struct PlayingScene {
    keyboard: Keyboard,
    waterfall: WaterfallRenderer,
    guidelines: GuidelineRenderer,

    player: MidiPlayer,
    rewind_controler: RewindController,
    bg_quad_pipeline: QuadPipeline,
    fg_quad_pipeline: QuadPipeline,
    toast_manager: ToastManager,

    top_bar: TopBar,
}

impl PlayingScene {
    pub fn new(target: &Target, song: Song) -> Self {
        let keyboard = Keyboard::new(target, song.config.clone());

        let keyboard_layout = keyboard.layout();

        let guidelines = GuidelineRenderer::new(
            keyboard_layout.clone(),
            *keyboard.pos(),
            target.config.vertical_guidelines,
            target.config.horizontal_guidelines,
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
            &target.gpu,
            &song.file.tracks,
            &hidden_tracks,
            &target.config,
            &target.transform,
            keyboard_layout.clone(),
        );

        let player = MidiPlayer::new(
            target.output_manager.connection().clone(),
            song,
            keyboard_layout.range.clone(),
        );
        waterfall.update(&target.gpu.queue, player.time_without_lead_in());

        Self {
            keyboard,
            guidelines,

            waterfall,
            player,
            rewind_controler: RewindController::new(),
            bg_quad_pipeline: QuadPipeline::new(&target.gpu, &target.transform),
            fg_quad_pipeline: QuadPipeline::new(&target.gpu, &target.transform),
            toast_manager: ToastManager::default(),
            top_bar: TopBar::default(),
        }
    }

    fn update_midi_player(&mut self, target: &Target, delta: Duration) -> f32 {
        if self.player.play_along().are_required_keys_pressed() {
            let delta = (delta / 10) * (target.config.speed_multiplier * 10.0) as u32;
            let midi_events = self.player.update(delta);
            self.keyboard.file_midi_events(&target.config, &midi_events);
        }

        self.player.time_without_lead_in() + target.config.playback_offset
    }
}

impl Scene for PlayingScene {
    fn resize(&mut self, target: &mut Target) {
        self.keyboard.resize(target);

        self.guidelines.set_layout(self.keyboard.layout().clone());
        self.guidelines.set_pos(*self.keyboard.pos());

        self.waterfall.resize(
            &target.gpu.queue,
            &target.config,
            self.keyboard.layout().clone(),
        );
    }

    fn update(&mut self, target: &mut Target, delta: Duration) {
        self.bg_quad_pipeline.clear();
        self.fg_quad_pipeline.clear();

        self.rewind_controler.update(&mut self.player, target);
        self.toast_manager.update(&mut target.text_renderer);

        let time = self.update_midi_player(target, delta);
        self.waterfall.update(&target.gpu.queue, time);
        self.guidelines.update(
            &mut self.bg_quad_pipeline,
            target.config.animation_speed,
            time,
        );
        self.keyboard
            .update(&mut self.fg_quad_pipeline, &mut target.text_renderer);

        TopBar::update(self, &target.window_state);

        self.bg_quad_pipeline.prepare(&target.gpu.queue);
        self.fg_quad_pipeline.prepare(&target.gpu.queue);
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

    fn window_event(&mut self, target: &mut Target, event: &WindowEvent) {
        self.rewind_controler
            .handle_window_event(target, event, &mut self.player);

        if self.rewind_controler.is_rewinding() {
            self.keyboard.reset_notes();
        }

        handle_back_button(target, event);
        handle_pause_button(&mut self.player, event);
        handle_settings_input(target, &mut self.toast_manager, &mut self.waterfall, event);
    }

    fn midi_event(&mut self, _target: &mut Target, _channel: u8, message: &MidiMessage) {
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

fn handle_back_button(target: &Target, event: &WindowEvent) {
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
        target.proxy.send_event(NeothesiaEvent::MainMenu).ok();
    }
}

fn handle_settings_input(
    target: &mut Target,
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
            let amount = if target.window_state.modifers_state.shift_key() {
                0.5
            } else {
                0.1
            };

            if key == NamedKey::ArrowUp {
                target.config.speed_multiplier += amount;
            } else {
                target.config.speed_multiplier -= amount;
                target.config.speed_multiplier = target.config.speed_multiplier.max(0.0);
            }

            toast_manager.speed_toast(target.config.speed_multiplier);
        }

        Key::Named(key @ (NamedKey::PageUp | NamedKey::PageDown)) => {
            let amount = if target.window_state.modifers_state.shift_key() {
                500.0
            } else {
                100.0
            };

            if key == NamedKey::PageUp {
                target.config.animation_speed += amount;
                // 0.0 is invalid speed, let's skip it and jump to positive
                if target.config.animation_speed == 0.0 {
                    target.config.animation_speed += amount;
                }
            } else {
                target.config.animation_speed -= amount;
                // 0.0 is invalid speed, let's skip it and jump to negative
                if target.config.animation_speed == 0.0 {
                    target.config.animation_speed -= amount;
                }
            }

            waterfall
                .pipeline()
                .set_speed(&target.gpu.queue, target.config.animation_speed);
            toast_manager.animation_speed_toast(target.config.animation_speed);
        }

        Key::Character(ref ch) if matches!(ch.as_str(), "_" | "-" | "+" | "=") => {
            let amount = if target.window_state.modifers_state.shift_key() {
                0.1
            } else {
                0.01
            };

            if matches!(ch.as_str(), "-" | "_") {
                target.config.playback_offset -= amount;
            } else {
                target.config.playback_offset += amount;
            }

            toast_manager.offset_toast(target.config.playback_offset);
        }

        _ => {}
    }
}
