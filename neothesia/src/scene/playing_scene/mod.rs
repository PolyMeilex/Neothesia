use midi_file::midly::MidiMessage;
use neothesia_core::render::{QuadInstance, QuadPipeline};
use std::time::Duration;
use wgpu_jumpstart::Color;
use winit::event::{KeyboardInput, WindowEvent};

use super::Scene;
use crate::{render::WaterfallRenderer, target::Target, NeothesiaEvent};

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

    player: MidiPlayer,
    rewind_controler: RewindController,
    quad_pipeline: QuadPipeline,
    toast_manager: ToastManager,
}

impl PlayingScene {
    pub fn new(target: &Target, midi_file: midi_file::Midi) -> Self {
        let keyboard = Keyboard::new(target);

        let keyboard_layout = keyboard.layout();

        let mut notes = WaterfallRenderer::new(
            &target.gpu,
            &midi_file,
            &target.config,
            &target.transform_uniform,
            keyboard_layout.clone(),
        );

        let player = MidiPlayer::new(target, midi_file, keyboard_layout.range.clone());
        notes.update(&target.gpu.queue, player.time_without_lead_in());

        Self {
            keyboard,

            notes,
            player,
            rewind_controler: RewindController::new(),
            quad_pipeline: QuadPipeline::new(&target.gpu, &target.transform_uniform),

            toast_manager: ToastManager::default(),
        }
    }

    fn update_progresbar(&mut self, target: &mut Target) {
        let size_x = target.window_state.logical_size.width * self.player.percentage();
        self.quad_pipeline.update_instance_buffer(
            &target.gpu.queue,
            vec![QuadInstance {
                position: [0.0, 0.0],
                size: [size_x, 5.0],
                color: Color::from_rgba8(56, 145, 255, 1.0).into_linear_rgba(),
                ..Default::default()
            }],
        );
    }
}

impl Scene for PlayingScene {
    fn resize(&mut self, target: &mut Target) {
        self.keyboard.resize(target);
        self.notes.resize(
            &target.gpu.queue,
            target.midi_file.as_ref().unwrap(),
            &target.config,
            self.keyboard.layout().clone(),
        );
    }

    fn update(&mut self, target: &mut Target, delta: Duration) {
        if self.player.play_along().are_required_keys_pressed() || !target.config.play_along {
            self.rewind_controler.update(&mut self.player, target);
            let midi_events = self.player.update(target, delta);
            self.keyboard.file_midi_events(&target.config, &midi_events);
        }

        self.update_progresbar(target);

        self.notes.update(
            &target.gpu.queue,
            self.player.time_without_lead_in() + target.config.playback_offset,
        );

        self.keyboard
            .update(&target.gpu.queue, target.text_renderer.glyph_brush());
        self.toast_manager.update(target);
    }

    fn render(&mut self, target: &mut Target, view: &wgpu::TextureView) {
        let mut render_pass = target
            .gpu
            .encoder
            .begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

        self.notes
            .render(&target.transform_uniform, &mut render_pass);

        self.keyboard
            .render(&target.transform_uniform, &mut render_pass);

        self.quad_pipeline
            .render(&target.transform_uniform, &mut render_pass)
    }

    fn window_event(&mut self, target: &mut Target, event: &WindowEvent) {
        use winit::event::WindowEvent::*;
        use winit::event::{ElementState, VirtualKeyCode};

        match &event {
            KeyboardInput { input, .. } => {
                self.rewind_controler
                    .handle_keyboard_input(&mut self.player, input);

                if self.rewind_controler.is_rewinding() {
                    self.keyboard.reset_notes();
                }

                settings_keyboard_input(target, &mut self.toast_manager, input, &mut self.notes);

                if input.state == ElementState::Released {
                    match input.virtual_keycode {
                        Some(VirtualKeyCode::Escape) => {
                            target.proxy.send_event(NeothesiaEvent::MainMenu).ok();
                        }
                        Some(VirtualKeyCode::Space) => {
                            self.player.pause_resume();
                        }
                        _ => {}
                    }
                }
            }
            MouseInput { state, button, .. } => {
                self.rewind_controler
                    .handle_mouse_input(&mut self.player, target, state, button);

                if self.rewind_controler.is_rewinding() {
                    self.keyboard.reset_notes();
                }
            }
            CursorMoved { position, .. } => {
                self.rewind_controler
                    .handle_cursor_moved(&mut self.player, target, position);
            }
            _ => {}
        }
    }

    fn midi_event(&mut self, _target: &mut Target, _channel: u8, message: &MidiMessage) {
        self.player
            .play_along_mut()
            .midi_event(midi_player::MidiEventSource::User, message);
        self.keyboard.user_midi_event(message);
    }
}

fn settings_keyboard_input(
    target: &mut Target,
    toast_manager: &mut ToastManager,
    input: &KeyboardInput,
    waterfall: &mut WaterfallRenderer,
) {
    use winit::event::{ElementState, VirtualKeyCode};

    if input.state != ElementState::Released {
        return;
    }

    let virtual_keycode = if let Some(virtual_keycode) = input.virtual_keycode {
        virtual_keycode
    } else {
        return;
    };

    match virtual_keycode {
        VirtualKeyCode::Up | VirtualKeyCode::Down => {
            let amount = if target.window_state.modifers_state.shift() {
                0.5
            } else {
                0.1
            };

            if virtual_keycode == VirtualKeyCode::Up {
                target.config.speed_multiplier += amount;
            } else {
                target.config.speed_multiplier -= amount;
                target.config.speed_multiplier = target.config.speed_multiplier.max(0.0);
            }

            toast_manager.speed_toast(target.config.speed_multiplier);
        }

        VirtualKeyCode::PageUp | VirtualKeyCode::PageDown => {
            let amount = if target.window_state.modifers_state.shift() {
                500.0
            } else {
                100.0
            };

            if virtual_keycode == VirtualKeyCode::PageUp {
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

        VirtualKeyCode::Minus | VirtualKeyCode::Plus | VirtualKeyCode::Equals => {
            let amount = if target.window_state.modifers_state.shift() {
                0.1
            } else {
                0.01
            };

            if virtual_keycode == VirtualKeyCode::Minus {
                target.config.playback_offset -= amount;
            } else {
                target.config.playback_offset += amount;
            }

            toast_manager.offset_toast(target.config.playback_offset);
        }

        _ => {}
    }
}
