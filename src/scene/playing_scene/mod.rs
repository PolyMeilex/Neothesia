mod keyboard;
mod keyboard_pipeline;

use keyboard::PianoKeyboard;

mod notes;
mod notes_pipeline;

use notes::Notes;

use super::{Scene, SceneEvent, SceneType};
use lib_midi::MidiNote;

use crate::{
    rectangle_pipeline::{RectangleInstance, RectanglePipeline},
    time_manager::Timer,
    wgpu_jumpstart::Color,
    MainState, Target,
};

use winit::event::WindowEvent;

pub struct PlayingScene {
    main_state: MainState,

    piano_keyboard: PianoKeyboard,
    notes: Notes,
    player: Player,
    rectangle_pipeline: RectanglePipeline,

    text_toast: Option<Toast>,
}

impl PlayingScene {
    pub fn new(target: &mut Target, mut main_state: MainState) -> Self {
        let piano_keyboard = PianoKeyboard::new(target);

        let mut notes = Notes::new(
            target,
            &piano_keyboard.all_keys,
            &main_state.midi_file.as_ref().unwrap(),
        );

        let player = Player::new(&mut main_state);
        notes.update(&mut target.gpu, player.time);

        Self {
            main_state,

            piano_keyboard,
            notes,
            player,
            rectangle_pipeline: RectanglePipeline::new(&target.gpu, &target.transform_uniform),

            text_toast: None,
        }
    }

    fn speed_toast(&mut self) {
        let s = format!(
            "Speed: {}",
            (self.main_state.config.speed_multiplier * 100.0).round() / 100.0
        );

        self.text_toast = Some(Toast::new(move |target| {
            let text = vec![wgpu_glyph::Text::new(&s)
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

    fn offset_toast(&mut self) {
        let s = format!(
            "Offset: {}",
            (self.main_state.config.playback_offset * 100.0).round() / 100.0
        );

        self.text_toast = Some(Toast::new(move |target| {
            let text = vec![wgpu_glyph::Text::new(&s)
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
}

impl Scene for PlayingScene {
    fn done(mut self: Box<Self>) -> MainState {
        self.player.clear(&mut self.main_state);

        self.main_state
    }

    fn scene_type(&self) -> SceneType {
        SceneType::Playing
    }
    fn start(&mut self) {
        self.player.start();
    }
    fn resize(&mut self, target: &mut Target) {
        self.piano_keyboard.resize(target);
        self.notes.resize(
            target,
            &self.piano_keyboard.all_keys,
            self.main_state.midi_file.as_ref().unwrap(),
        );
    }
    fn update(&mut self, target: &mut Target) -> SceneEvent {
        let (window_w, _) = {
            let winit::dpi::LogicalSize { width, height } = target.window.state.logical_size;
            (width, height)
        };

        let notes_on = self.player.update(&mut self.main_state);

        let size_x = window_w * self.player.percentage;

        self.rectangle_pipeline.update_instance_buffer(
            &mut target.gpu.encoder,
            &target.gpu.device,
            vec![RectangleInstance {
                position: [0.0, 0.0],
                size: [size_x, 5.0],
                color: Color::from_rgba8(56, 145, 255, 1.0).into_linear_rgba(),
            }],
        );

        let pos = &target.window.state.cursor_logical_position;
        if pos.y < 20.0
            && target
                .window
                .state
                .mouse_is_pressed(winit::event::MouseButton::Left)
        {
            let x = pos.x;
            let p = x / window_w;
            log::debug!("Progressbar Clicked: x:{},p:{}", x, p);
            self.player.set_percentage_time(&mut self.main_state, p);

            if !self.player.rewind_controler.is_rewinding() {
                self.player.start_rewind(RewindControler::Mouse {
                    was_paused: self.player.timer.paused,
                });
            }
        } else {
            if let RewindControler::Mouse { .. } = self.player.rewind_controler {
                self.player.stop_rewind();
            }
        }

        self.piano_keyboard
            .update_notes_state(&mut target.gpu, notes_on);
        self.notes.update(
            &mut target.gpu,
            self.player.time + self.main_state.config.playback_offset,
        );

        // Toasts
        {
            if let Some(mut toast) = self.text_toast.take() {
                self.text_toast = if toast.draw(target) {
                    Some(toast)
                } else {
                    None
                };
            }
        }

        SceneEvent::None
    }
    fn render(&mut self, target: &mut Target, frame: &wgpu::SwapChainFrame) {
        let transform_uniform = &target.transform_uniform;
        let encoder = &mut target.gpu.encoder;
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &frame.output.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });

            self.notes.render(&transform_uniform, &mut render_pass);

            self.piano_keyboard
                .render(&transform_uniform, &mut render_pass);

            self.rectangle_pipeline
                .render(&target.transform_uniform, &mut render_pass)
        }
    }
    fn window_event(&mut self, target: &mut Target, event: &WindowEvent) -> SceneEvent {
        match &event {
            winit::event::WindowEvent::KeyboardInput { input, .. } => match input.virtual_keycode {
                Some(winit::event::VirtualKeyCode::Escape) => {
                    if let winit::event::ElementState::Released = input.state {
                        return SceneEvent::GoBack;
                    }
                }
                Some(winit::event::VirtualKeyCode::Space) => {
                    if let winit::event::ElementState::Released = input.state {
                        self.player.pause_resume(&mut self.main_state);
                    }
                }
                Some(winit::event::VirtualKeyCode::Left) => {
                    if let winit::event::ElementState::Pressed = input.state {
                        let speed = if target.window.state.modifers_state.shift() {
                            -0.0001 * 50.0
                        } else {
                            -0.0001
                        };

                        if !self.player.rewind_controler.is_rewinding() {
                            self.player.start_rewind(RewindControler::Keyboard {
                                speed,
                                was_paused: self.player.timer.paused,
                            });
                        }
                    } else {
                        self.player.stop_rewind();
                    }
                }
                Some(winit::event::VirtualKeyCode::Right) => {
                    if let winit::event::ElementState::Pressed = input.state {
                        let speed = if target.window.state.modifers_state.shift() {
                            0.0001 * 50.0
                        } else {
                            0.0001
                        };

                        if !self.player.rewind_controler.is_rewinding() {
                            self.player.start_rewind(RewindControler::Keyboard {
                                speed,
                                was_paused: self.player.timer.paused,
                            });
                        }
                    } else {
                        self.player.stop_rewind();
                    }
                }
                Some(winit::event::VirtualKeyCode::Up) => {
                    if let winit::event::ElementState::Released = input.state {
                        if target.window.state.modifers_state.shift() {
                            self.main_state.config.speed_multiplier += 0.5;
                        } else {
                            self.main_state.config.speed_multiplier += 0.1;
                        }

                        self.player
                            .set_percentage_time(&mut self.main_state, self.player.percentage);

                        self.speed_toast();
                    }
                }
                Some(winit::event::VirtualKeyCode::Down) => {
                    if let winit::event::ElementState::Released = input.state {
                        let new = if target.window.state.modifers_state.shift() {
                            self.main_state.config.speed_multiplier - 0.5
                        } else {
                            self.main_state.config.speed_multiplier - 0.1
                        };

                        if new > 0.0 {
                            self.main_state.config.speed_multiplier = new;
                            self.player
                                .set_percentage_time(&mut self.main_state, self.player.percentage);
                        }

                        self.speed_toast();
                    }
                }
                Some(winit::event::VirtualKeyCode::Minus) => {
                    if let winit::event::ElementState::Released = input.state {
                        if target.window.state.modifers_state.shift() {
                            self.main_state.config.playback_offset -= 0.1;
                        } else {
                            self.main_state.config.playback_offset -= 0.01;
                        }

                        self.offset_toast();
                    }
                }
                Some(winit::event::VirtualKeyCode::Plus)
                | Some(winit::event::VirtualKeyCode::Equals) => {
                    if let winit::event::ElementState::Released = input.state {
                        if target.window.state.modifers_state.shift() {
                            self.main_state.config.playback_offset += 0.1;
                        } else {
                            self.main_state.config.playback_offset += 0.01;
                        }

                        self.offset_toast();
                    }
                }
                _ => {}
            },
            _ => {}
        }

        SceneEvent::None
    }
}

use std::collections::HashMap;

struct Player {
    midi_first_note_start: f32,
    midi_last_note_end: f32,
    active_notes: HashMap<usize, MidiNote>,
    timer: Timer,
    percentage: f32,
    time: f32,

    rewind_controler: RewindControler,
}

impl Player {
    fn new(main_state: &mut MainState) -> Self {
        let midi_file = main_state.midi_file.as_ref().unwrap();

        let midi_first_note_start = if let Some(note) = midi_file.merged_track.notes.first() {
            note.start
        } else {
            0.0
        };
        let midi_last_note_end = if let Some(note) = midi_file.merged_track.notes.last() {
            note.start + note.duration
        } else {
            0.0
        };

        let mut player = Self {
            midi_first_note_start,
            midi_last_note_end,
            active_notes: HashMap::new(),
            timer: Timer::new(),
            percentage: 0.0,
            time: 0.0,

            rewind_controler: RewindControler::None,
        };
        player.update(main_state);

        player
    }
    fn start(&mut self) {
        self.timer.start();
    }

    fn update(&mut self, main_state: &mut MainState) -> [(bool, usize); 88] {
        if let RewindControler::Keyboard { speed, .. } = self.rewind_controler {
            let p = self.percentage + speed;
            self.set_percentage_time(main_state, p);
        }

        self.timer.update();
        let raw_time = self.timer.get_elapsed() / 1000.0 * main_state.config.speed_multiplier;
        self.percentage = raw_time / (self.midi_last_note_end + 3.0);
        self.time = raw_time + self.midi_first_note_start - 3.0;

        if self.timer.paused {
            return [(false, 0); 88];
        };

        let mut notes_state: [(bool, usize); 88] = [(false, 0); 88];

        let filtered: Vec<&lib_midi::MidiNote> = main_state
            .midi_file
            .as_ref()
            .unwrap()
            .merged_track
            .notes
            .iter()
            .filter(|n| n.start <= self.time && n.start + n.duration + 0.5 > self.time)
            .collect();

        let output_manager = &mut main_state.output_manager;
        for n in filtered {
            use std::collections::hash_map::Entry;

            if n.start + n.duration >= self.time {
                if n.note >= 21 && n.note <= 108 {
                    notes_state[n.note as usize - 21] = (true, n.track_id);
                }

                if let Entry::Vacant(_e) = self.active_notes.entry(n.id) {
                    self.active_notes.insert(n.id, n.clone());
                    output_manager.note_on(n.ch, n.note, n.vel);
                }
            } else if let Entry::Occupied(_e) = self.active_notes.entry(n.id) {
                self.active_notes.remove(&n.id);
                output_manager.note_off(n.ch, n.note);
            }
        }

        notes_state
    }

    fn pause_resume(&mut self, main_state: &mut MainState) {
        self.clear(main_state);
        self.timer.pause_resume();
    }

    fn start_rewind(&mut self, controler: RewindControler) {
        self.timer.pause();
        self.rewind_controler = controler;
    }
    fn stop_rewind(&mut self) {
        let controler = std::mem::replace(&mut self.rewind_controler, RewindControler::None);

        let was_paused = match controler {
            RewindControler::Keyboard { was_paused, .. } => was_paused,
            RewindControler::Mouse { was_paused } => was_paused,
            RewindControler::None => return,
        };

        if !was_paused {
            self.timer.resume();
        }
    }

    fn set_time(&mut self, main_state: &mut MainState, time: f32) {
        self.timer.set_time(time * 1000.0);
        self.clear(main_state);
    }

    fn set_percentage_time(&mut self, main_state: &mut MainState, p: f32) {
        self.set_time(
            main_state,
            p * (self.midi_last_note_end + 3.0) / main_state.config.speed_multiplier,
        );
    }

    fn clear(&mut self, main_state: &mut MainState) {
        for (_id, n) in self.active_notes.iter() {
            main_state.output_manager.note_off(n.ch, n.note);
        }
        self.active_notes.clear();
    }
}

enum RewindControler {
    Keyboard { speed: f32, was_paused: bool },
    Mouse { was_paused: bool },
    None,
}
impl RewindControler {
    fn is_rewinding(&self) -> bool {
        match self {
            RewindControler::None => false,
            _ => true,
        }
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
