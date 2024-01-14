use midi_file::midly::MidiMessage;
use neothesia_core::render::{GuidelineRenderer, QuadInstance, QuadPipeline};
use std::time::Duration;
use wgpu_jumpstart::{Color, TransformUniform, Uniform};
use winit::{
    event::{ElementState, KeyEvent, MouseButton, WindowEvent},
    keyboard::{Key, NamedKey},
};

use super::Scene;
use crate::{
    render::WaterfallRenderer, song::Song, target::Target, utils::window::WindowState,
    NeothesiaEvent,
};

mod keyboard;
use keyboard::Keyboard;

mod midi_player;
use midi_player::MidiPlayer;

mod rewind_controller;
use rewind_controller::RewindController;

mod toast_manager;
use toast_manager::ToastManager;

pub struct PlayingScene {
    keyboard: Keyboard,
    notes: WaterfallRenderer,
    guidelines: GuidelineRenderer,

    player: MidiPlayer,
    rewind_controler: RewindController,
    bg_quad_pipeline: QuadPipeline,
    quad_pipeline: QuadPipeline,
    toast_manager: ToastManager,
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
            song.file.mesures.clone(),
        );

        let hidden_tracks: Vec<usize> = song
            .config
            .tracks
            .iter()
            .filter(|t| !t.visible)
            .map(|t| t.track_id)
            .collect();

        let mut notes = WaterfallRenderer::new(
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
        notes.update(&target.gpu.queue, player.time_without_lead_in());

        Self {
            keyboard,
            guidelines,

            notes,
            player,
            rewind_controler: RewindController::new(),
            bg_quad_pipeline: QuadPipeline::new(&target.gpu, &target.transform),
            quad_pipeline: QuadPipeline::new(&target.gpu, &target.transform),
            toast_manager: ToastManager::default(),
        }
    }

    fn update_progresbar(&mut self, window_state: &WindowState) {
        let size_x = window_state.logical_size.width * self.player.percentage();
        self.quad_pipeline.instances().push(QuadInstance {
            position: [0.0, 0.0],
            size: [size_x, 5.0],
            color: Color::from_rgba8(56, 145, 255, 1.0).into_linear_rgba(),
            ..Default::default()
        });
    }
}

impl Scene for PlayingScene {
    fn resize(&mut self, target: &mut Target) {
        self.keyboard.resize(target);

        self.guidelines.set_layout(self.keyboard.layout().clone());
        self.guidelines.set_pos(*self.keyboard.pos());

        self.notes.resize(
            &target.gpu.queue,
            &target.config,
            self.keyboard.layout().clone(),
        );
    }

    fn update(&mut self, target: &mut Target, delta: Duration) {
        self.rewind_controler.update(&mut self.player, target);

        if self.player.play_along().are_required_keys_pressed() {
            let delta = (delta / 10) * (target.config.speed_multiplier * 10.0) as u32;
            let midi_events = self.player.update(delta);
            self.keyboard.file_midi_events(&target.config, &midi_events);
        }

        self.toast_manager.update(&mut target.text_renderer);

        let time = self.player.time_without_lead_in() + target.config.playback_offset;

        self.bg_quad_pipeline.clear();
        self.guidelines.update(
            &mut self.bg_quad_pipeline,
            target.config.animation_speed,
            time,
        );
        self.bg_quad_pipeline.prepare(&target.gpu.queue);

        self.notes.update(&target.gpu.queue, time);

        self.quad_pipeline.clear();

        self.keyboard
            .update(&mut self.quad_pipeline, &mut target.text_renderer);

        self.update_progresbar(&target.window_state);

        self.quad_pipeline.prepare(&target.gpu.queue);
    }

    fn render<'pass>(
        &'pass mut self,
        transform: &'pass Uniform<TransformUniform>,
        rpass: &mut wgpu::RenderPass<'pass>,
    ) {
        self.bg_quad_pipeline.render(transform, rpass);
        self.notes.render(transform, rpass);
        self.quad_pipeline.render(transform, rpass);
    }

    fn window_event(&mut self, target: &mut Target, event: &WindowEvent) {
        self.rewind_controler
            .handle_window_event(target, event, &mut self.player);

        if self.rewind_controler.is_rewinding() {
            self.keyboard.reset_notes();
        }

        handle_back_button(target, event);
        handle_pause_button(&mut self.player, event);
        handle_settings_input(target, &mut self.toast_manager, &mut self.notes, event);
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
            } else {
                target.config.animation_speed -= amount;
                target.config.animation_speed = target.config.animation_speed.max(100.0);
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
