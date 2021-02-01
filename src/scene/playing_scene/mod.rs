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
    piano_keyboard: PianoKeyboard,
    notes: Notes,
    player: Player,
    rectangle_pipeline: RectanglePipeline,

    text_toast: Option<Toast>,
}

impl PlayingScene {
    pub fn new(target: &mut Target) -> Self {
        let piano_keyboard = PianoKeyboard::new(target);

        let mut notes = Notes::new(target, &piano_keyboard.all_keys);

        let player = Player::new(&mut target.state);
        notes.update(target, player.time);

        Self {
            piano_keyboard,
            notes,
            player,
            rectangle_pipeline: RectanglePipeline::new(&target.gpu, &target.transform_uniform),

            text_toast: None,
        }
    }

    fn speed_toast(&mut self, target: &mut Target) {
        let s = format!(
            "Speed: {}",
            (target.state.config.speed_multiplier * 100.0).round() / 100.0
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

    fn offset_toast(&mut self, target: &mut Target) {
        let s = format!(
            "Offset: {}",
            (target.state.config.playback_offset * 100.0).round() / 100.0
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
    fn done(mut self: Box<Self>, target: &mut Target) {
        self.player.clear(&mut target.state);
    }

    fn scene_type(&self) -> SceneType {
        SceneType::Playing
    }
    fn start(&mut self) {
        self.player.start();
    }
    fn resize(&mut self, target: &mut Target) {
        self.piano_keyboard.resize(target);
        self.notes.resize(target, &self.piano_keyboard.all_keys);
    }
    fn update(&mut self, target: &mut Target) -> SceneEvent {
        let (window_w, _) = {
            let winit::dpi::LogicalSize { width, height } = target.window.state.logical_size;
            (width, height)
        };

        let notes_on = self.player.update(&mut target.state);

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
            self.player.set_percentage_time(&mut target.state, p);

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

        self.piano_keyboard.update_notes_state(target, notes_on);
        self.notes.update(
            target,
            self.player.time + target.state.config.playback_offset,
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
                        self.player.pause_resume(&mut target.state);
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
                            target.state.config.speed_multiplier += 0.5;
                        } else {
                            target.state.config.speed_multiplier += 0.1;
                        }

                        self.player
                            .set_percentage_time(&mut target.state, self.player.percentage);

                        self.speed_toast(target);
                    }
                }
                Some(winit::event::VirtualKeyCode::Down) => {
                    if let winit::event::ElementState::Released = input.state {
                        let new = if target.window.state.modifers_state.shift() {
                            target.state.config.speed_multiplier - 0.5
                        } else {
                            target.state.config.speed_multiplier - 0.1
                        };

                        if new > 0.0 {
                            target.state.config.speed_multiplier = new;
                            self.player
                                .set_percentage_time(&mut target.state, self.player.percentage);
                        }

                        self.speed_toast(target);
                    }
                }
                Some(winit::event::VirtualKeyCode::Minus) => {
                    if let winit::event::ElementState::Released = input.state {
                        if target.window.state.modifers_state.shift() {
                            target.state.config.playback_offset -= 0.1;
                        } else {
                            target.state.config.playback_offset -= 0.01;
                        }

                        self.offset_toast(target);
                    }
                }
                Some(winit::event::VirtualKeyCode::Plus)
                | Some(winit::event::VirtualKeyCode::Equals) => {
                    if let winit::event::ElementState::Released = input.state {
                        if target.window.state.modifers_state.shift() {
                            target.state.config.playback_offset += 0.1;
                        } else {
                            target.state.config.playback_offset += 0.01;
                        }

                        self.offset_toast(target);
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
    #[cfg(feature = "play_along")]
    play_along_controler: Option<PlayAlongControler>,
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

        #[cfg(feature = "play_along")]
        let play_along_controler = if main_state.config.play_along {
            PlayAlongControler::new()
        } else {
            None
        };

        let mut player = Self {
            midi_first_note_start,
            midi_last_note_end,
            active_notes: HashMap::new(),
            timer: Timer::new(),
            percentage: 0.0,
            time: 0.0,

            rewind_controler: RewindControler::None,
            #[cfg(feature = "play_along")]
            play_along_controler,
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

        let mut notes_state: [(bool, usize); 88] = [(false, 0); 88];

        #[cfg(feature = "play_along")]
        if let Some(controler) = &mut self.play_along_controler {
            controler.update(main_state, &mut notes_state, &mut self.timer);
        }

        if self.timer.paused {
            return notes_state;
        };

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
                if n.note >= 21 && n.note <= 108 && n.ch != 9 {
                    notes_state[n.note as usize - 21] = (true, n.track_id);
                }

                if let Entry::Vacant(_e) = self.active_notes.entry(n.id) {
                    self.active_notes.insert(n.id, n.clone());

                    #[cfg(feature = "play_along")]
                    if let Some(controler) = &mut self.play_along_controler {
                        controler.require_note(&mut self.timer, &n);
                    } else {
                        output_manager.note_on(n.ch, n.note, n.vel);
                    }

                    #[cfg(not(feature = "play_along"))]
                    output_manager.note_on(n.ch, n.note, n.vel);
                }
            } else if let Entry::Occupied(_e) = self.active_notes.entry(n.id) {
                self.active_notes.remove(&n.id);

                if !main_state.config.play_along {
                    output_manager.note_off(n.ch, n.note);
                }
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

        #[cfg(feature = "play_along")]
        if let Some(controler) = &mut self.play_along_controler {
            controler.clear();
        }
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

#[cfg(feature = "play_along")]
use std::sync::{mpsc, Arc, Mutex};

#[cfg(feature = "play_along")]
struct PlayAlongControler {
    _midi_in_conn: midir::MidiInputConnection<()>,
    midi_in_rec: mpsc::Receiver<(bool, u8, u8)>,

    input_pressed_keys: [bool; 88],
    required_notes: Arc<Mutex<HashMap<u8, MidiNote>>>,
    waiting_for_note: bool,
}

#[cfg(feature = "play_along")]
impl PlayAlongControler {
    fn new() -> Option<Self> {
        let input_pressed_keys = [false; 88];
        let required_notes = Arc::new(Mutex::new(HashMap::new()));

        let (tx, midi_in_rec) = mpsc::channel();

        let _midi_in_conn = {
            let midi_in = midir::MidiInput::new("Neothesia-in").unwrap();
            let in_ports = midi_in.ports();

            use std::io::{stdin, stdout, Write};

            let in_port = match in_ports.len() {
                0 => return None,
                1 => {
                    println!(
                        "Choosing the only available input port: {}",
                        midi_in.port_name(&in_ports[0]).unwrap()
                    );
                    &in_ports[0]
                }
                _ => {
                    println!("\nAvailable input ports:");
                    for (i, p) in in_ports.iter().enumerate() {
                        println!("{}: {}", i, midi_in.port_name(p).unwrap());
                    }
                    print!("Please select input port: ");
                    stdout().flush().unwrap();
                    let mut input = String::new();
                    stdin().read_line(&mut input).unwrap();
                    in_ports
                        .get(input.trim().parse::<usize>().unwrap())
                        .ok_or("invalid input port selected")
                        .unwrap()
                }
            };

            let required_notes = required_notes.clone();

            midi_in
                .connect(
                    in_port,
                    "neothesia-read-input",
                    move |_, message, _| {
                        if message.len() == 3 {
                            let note = message[1];
                            if note >= 21 && note <= 108 {
                                if message[0] == 128 || message[2] == 0 {
                                    tx.send((false, message[1], message[2])).unwrap();
                                } else if message[0] == 144 {
                                    required_notes.lock().unwrap().remove(&note);
                                    tx.send((true, message[1], message[2])).unwrap();
                                }
                            }
                        }
                    },
                    (),
                )
                .unwrap()
        };

        Some(Self {
            _midi_in_conn,
            midi_in_rec,

            input_pressed_keys,
            required_notes,
            waiting_for_note: false,
        })
    }

    fn update(
        &mut self,
        main_state: &mut MainState,
        notes_state: &mut [(bool, usize); 88],
        timer: &mut Timer,
    ) {
        for (id, is) in self.input_pressed_keys.iter().enumerate() {
            notes_state[id] = (*is, 0);
        }

        if let Ok(event) = self.midi_in_rec.try_recv() {
            if event.0 {
                self.input_pressed_keys[event.1 as usize - 21] = true;
                main_state.output_manager.note_on(0, event.1, event.2)
            } else {
                self.input_pressed_keys[event.1 as usize - 21] = false;
                main_state.output_manager.note_off(0, event.1)
            }
        }
        if self.required_notes.lock().unwrap().len() == 0 && self.waiting_for_note == true {
            self.waiting_for_note = false;
            timer.resume();
        }
    }

    fn require_note(&mut self, timer: &mut Timer, n: &MidiNote) {
        if n.note >= 21 && n.note <= 108 && n.ch != 9 {
            self.required_notes
                .lock()
                .unwrap()
                .insert(n.note, n.clone());
            self.waiting_for_note = true;
            timer.pause();
        }
    }

    fn clear(&mut self) {
        self.required_notes.lock().unwrap().clear();
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
