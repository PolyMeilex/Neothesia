mod keyboard;

use std::time::Duration;

use keyboard::PianoKeyboard;

mod notes;

mod midi_player;
use midi_player::{rewind_controler, MidiPlayer};

use notes::Notes;

use super::{Scene, SceneType};

use crate::{midi_event::MidiEvent, target::Target, NeothesiaEvent};
use neothesia_pipelines::quad::{QuadInstance, QuadPipeline};
use wgpu_jumpstart::Color;

use winit::event::WindowEvent;

pub struct PlayingScene {
    piano_keyboard: PianoKeyboard,
    notes: Notes,
    player: MidiPlayer,
    rectangle_pipeline: QuadPipeline,

    text_toast: Option<Toast>,
}

impl PlayingScene {
    pub fn new(target: &mut Target) -> Self {
        let piano_keyboard = PianoKeyboard::new(target);

        let mut notes = Notes::new(target, piano_keyboard.keys());

        let player = MidiPlayer::new(target);
        notes.update(target, player.time_without_lead_in());

        Self {
            piano_keyboard,
            notes,
            player,
            rectangle_pipeline: QuadPipeline::new(&target.gpu, &target.transform_uniform),

            text_toast: None,
        }
    }

    fn toast(&mut self, text: String) {
        self.text_toast = Some(Toast::new(move |target| {
            let text = vec![wgpu_glyph::Text::new(&text)
                .with_color([1.0, 1.0, 1.0, 1.0])
                .with_scale(20.0)];

            target.text_renderer.queue_text(wgpu_glyph::Section {
                text,
                screen_position: (0.0, 20.0),
                layout: wgpu_glyph::Layout::Wrap {
                    line_breaker: Default::default(),
                    h_align: wgpu_glyph::HorizontalAlign::Left,
                    v_align: wgpu_glyph::VerticalAlign::Top,
                },
                ..Default::default()
            });
        }));
    }

    fn speed_toast(&mut self, target: &mut Target) {
        let s = format!(
            "Speed: {}",
            (target.config.speed_multiplier * 100.0).round() / 100.0
        );

        self.toast(s);
    }

    fn offset_toast(&mut self, target: &mut Target) {
        let s = format!(
            "Offset: {}",
            (target.config.playback_offset * 100.0).round() / 100.0
        );

        self.toast(s);
    }

    #[cfg(feature = "record")]
    pub fn playback_progress(&self) -> f32 {
        self.player.percentage() * 100.0
    }
}

impl Scene for PlayingScene {
    fn scene_type(&self) -> SceneType {
        SceneType::Playing
    }

    fn start(&mut self) {
        self.player.start();
    }

    fn done(mut self: Box<Self>, target: &mut Target) {
        self.player.pause(&mut target.output_manager);
    }

    fn resize(&mut self, target: &mut Target) {
        self.piano_keyboard.resize(target.window.state.logical_size);
        self.notes.resize(target, self.piano_keyboard.keys());
    }

    fn update(&mut self, target: &mut Target, delta: Duration) {
        let (window_w, _) = {
            let winit::dpi::LogicalSize { width, height } = target.window.state.logical_size;
            (width, height)
        };

        let midi_events = self.player.update(target, delta);

        let size_x = window_w * self.player.percentage();

        self.rectangle_pipeline.update_instance_buffer(
            &target.gpu.queue,
            vec![QuadInstance {
                position: [0.0, 0.0],
                size: [size_x, 5.0],
                color: Color::from_rgba8(56, 145, 255, 1.0).into_linear_rgba(),
                ..Default::default()
            }],
        );

        if let Some(midi_events) = midi_events {
            self.piano_keyboard
                .update_note_events(&target.config, &midi_events);
        } else {
            self.piano_keyboard.reset_notes();
        }

        self.notes.update(
            target,
            self.player.time_without_lead_in() + target.config.playback_offset,
        );

        self.piano_keyboard.update(target);

        // Toast
        if let Some(mut toast) = self.text_toast.take() {
            self.text_toast = if toast.draw(target) {
                Some(toast)
            } else {
                None
            };
        }
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

        self.piano_keyboard
            .render(&target.transform_uniform, &mut render_pass);

        self.rectangle_pipeline
            .render(&target.transform_uniform, &mut render_pass)
    }

    fn window_event(&mut self, target: &mut Target, event: &WindowEvent) {
        use winit::event::WindowEvent::{CursorMoved, KeyboardInput, MouseInput};
        use winit::event::{ElementState, VirtualKeyCode};

        match &event {
            KeyboardInput { input, .. } => {
                rewind_controler::rewind_keyboard_input(
                    &mut self.player,
                    &mut target.output_manager,
                    input,
                );

                if let Some(virtual_keycode) = input.virtual_keycode {
                    match virtual_keycode {
                        VirtualKeyCode::Escape => {
                            if let ElementState::Released = input.state {
                                target.proxy.send_event(NeothesiaEvent::GoBack).unwrap();
                            }
                        }
                        VirtualKeyCode::Space => {
                            if let ElementState::Released = input.state {
                                self.player.pause_resume(&mut target.output_manager);
                            }
                        }
                        VirtualKeyCode::Up => {
                            if let winit::event::ElementState::Released = input.state {
                                if target.window.state.modifers_state.shift() {
                                    target.config.speed_multiplier += 0.5;
                                } else {
                                    target.config.speed_multiplier += 0.1;
                                }

                                self.speed_toast(target);
                            }
                        }
                        VirtualKeyCode::Down => {
                            if let winit::event::ElementState::Released = input.state {
                                target.config.speed_multiplier =
                                    if target.window.state.modifers_state.shift() {
                                        target.config.speed_multiplier - 0.5
                                    } else {
                                        target.config.speed_multiplier - 0.1
                                    }
                                    .max(0.0);

                                self.speed_toast(target);
                            }
                        }
                        VirtualKeyCode::Minus => {
                            if let winit::event::ElementState::Released = input.state {
                                if target.window.state.modifers_state.shift() {
                                    target.config.playback_offset -= 0.1;
                                } else {
                                    target.config.playback_offset -= 0.01;
                                }

                                self.offset_toast(target);
                            }
                        }
                        VirtualKeyCode::Plus | VirtualKeyCode::Equals => {
                            if let winit::event::ElementState::Released = input.state {
                                if target.window.state.modifers_state.shift() {
                                    target.config.playback_offset += 0.1;
                                } else {
                                    target.config.playback_offset += 0.01;
                                }

                                self.offset_toast(target);
                            }
                        }
                        _ => {}
                    }
                }
            }
            MouseInput { state, button, .. } => {
                rewind_controler::rewind_mouse_input(&mut self.player, target, state, button);
            }
            CursorMoved { position, .. } => {
                rewind_controler::rewind_handle_cursor_moved(&mut self.player, target, position);
            }
            _ => {}
        }
    }

    fn midi_event(&mut self, _target: &mut Target, _event: &MidiEvent) {
        // TODO
    }
}

struct Toast {
    start_time: std::time::Instant,
    inner_draw: Box<dyn Fn(&mut Target)>,
}

impl Toast {
    fn new(draw: impl Fn(&mut Target) + 'static) -> Self {
        Self {
            start_time: std::time::Instant::now(),
            inner_draw: Box::new(draw),
        }
    }

    fn draw(&mut self, target: &mut Target) -> bool {
        let time = self.start_time.elapsed().as_secs();

        if time < 1 {
            (*self.inner_draw)(target);

            true
        } else {
            false
        }
    }
}
