use super::{
    super::{Scene, SceneEvent, SceneType},
    keyboard::PianoKeyboard,
    notes::Notes,
};

use crate::{
    rectangle_pipeline::{RectangleInstance, RectanglePipeline},
    time_manager::Timer,
    ui::Ui,
    wgpu_jumpstart::{Color, Gpu},
    MainState,
};

use winit::event::WindowEvent;

pub struct PlayingScene {
    piano_keyboard: PianoKeyboard,
    notes: Notes,
    player: Player,
    rectangle_pipeline: RectanglePipeline,
}

impl PlayingScene {
    pub fn new(gpu: &mut Gpu, state: &mut MainState, port: MidiPortInfo) -> Self {
        let piano_keyboard = PianoKeyboard::new(state, gpu);
        let mut notes = Notes::new(
            state,
            gpu,
            &piano_keyboard.all_keys,
            &state
                .midi_file
                .clone()
                .expect("Expeced Midi File, no mifi file selected"),
        );

        let player = Player::new(state.midi_file.clone().unwrap(), port);
        notes.update(gpu, player.time);

        Self {
            piano_keyboard,
            notes,
            player,
            rectangle_pipeline: RectanglePipeline::new(&state, &gpu.device),
        }
    }
}

impl Scene for PlayingScene {
    fn scene_type(&self) -> SceneType {
        SceneType::Playing
    }
    fn start(&mut self) {
        self.player.start();
    }
    fn resize(&mut self, state: &mut MainState, gpu: &mut Gpu) {
        self.piano_keyboard.resize(state, gpu);
        self.notes
            .resize(state, gpu, &self.piano_keyboard.all_keys, &self.player.midi);
    }
    fn update(&mut self, state: &mut MainState, gpu: &mut Gpu, ui: &mut Ui) -> SceneEvent {
        let notes_on = self.player.update();

        let size_x = state.window_size.0 * self.player.percentage;
        ui.queue_rectangle(RectangleInstance {
            position: [size_x / 2.0, 0.0],
            size: [size_x, 10.0],
            color: Color::from_rgba8(56, 145, 255, 1.0).into_linear_rgba(),
        });

        if state.mouse_pos.1 < 20.0 && state.mouse_pressed {
            let x = state.mouse_pos.0;
            let p = x / state.window_size.0;
            log::debug!("Progressbar Clicked: x:{},p:{}", x, p);
            self.player.set_time(p * self.player.midi_last_note_end)
        }

        self.piano_keyboard.update_notes_state(gpu, notes_on);
        self.notes.update(gpu, self.player.time);

        SceneEvent::None
    }
    fn render(&mut self, state: &mut MainState, gpu: &mut Gpu, frame: &wgpu::SwapChainFrame) {
        self.notes.render(state, gpu, frame);
        self.piano_keyboard.render(state, gpu, frame);

        let encoder = &mut gpu.encoder;
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
            self.rectangle_pipeline.render(state, &mut render_pass)
        }
    }
    fn window_event(&mut self, _main_state: &mut MainState, event: &WindowEvent) -> SceneEvent {
        match &event {
            winit::event::WindowEvent::KeyboardInput { input, .. } => match input.virtual_keycode {
                Some(winit::event::VirtualKeyCode::Escape) => {
                    if let winit::event::ElementState::Released = input.state {
                        return SceneEvent::GoBack;
                    }
                }
                Some(winit::event::VirtualKeyCode::Space) => {
                    if let winit::event::ElementState::Released = input.state {
                        self.player.pause_resume();
                    }
                }
                _ => {}
            },
            _ => {}
        }

        SceneEvent::None
    }
}

use crate::midi_device::MidiPortInfo;
use std::{collections::HashMap, sync::Arc};
struct Player {
    midi: Arc<lib_midi::Midi>,
    midi_first_note_start: f32,
    midi_last_note_end: f32,
    midi_device: crate::midi_device::MidiDevicesManager,
    active_notes: HashMap<usize, u8>,
    timer: Timer,
    percentage: f32,
    time: f32,
    active: bool,
}

impl Player {
    fn new(midi: Arc<lib_midi::Midi>, port: MidiPortInfo) -> Self {
        let mut midi_device = crate::midi_device::MidiDevicesManager::new();

        log::info!("{:?}", midi_device.get_outs());

        midi_device.connect_out(port);

        let midi_first_note_start = if let Some(note) = midi.merged_track.notes.first() {
            note.start
        } else {
            0.0
        };
        let midi_last_note_end = if let Some(note) = midi.merged_track.notes.last() {
            note.start + note.duration
        } else {
            0.0
        };

        let mut player = Self {
            midi,
            midi_first_note_start,
            midi_last_note_end,
            midi_device,
            active_notes: HashMap::new(),
            timer: Timer::new(),
            percentage: 0.0,
            time: 0.0,
            active: true,
        };
        player.update();
        player.active = false;

        player
    }
    fn start(&mut self) {
        self.timer.start();
        self.active = true;
    }
    fn update(&mut self) -> [(bool, usize); 88] {
        if !self.active {
            return [(false, 0); 88];
        };
        self.timer.update();
        let raw_time = self.timer.get_elapsed() / 1000.0;
        self.percentage = raw_time / self.midi_last_note_end;
        self.time = raw_time + self.midi_first_note_start - 3.0;

        let mut notes_state: [(bool, usize); 88] = [(false, 0); 88];

        let filtered: Vec<&lib_midi::MidiNote> = self
            .midi
            .merged_track
            .notes
            .iter()
            .filter(|n| n.start <= self.time && n.start + n.duration + 0.5 > self.time)
            .collect();

        let midi_out = &mut self.midi_device;
        for n in filtered {
            use std::collections::hash_map::Entry;

            if n.start + n.duration >= self.time {
                if n.note >= 21 && n.note <= 108 {
                    notes_state[n.note as usize - 21] = (true, n.track_id);
                }

                if let Entry::Vacant(_e) = self.active_notes.entry(n.id) {
                    self.active_notes.insert(n.id, n.note);
                    midi_out.send(&[0x90, n.note, n.vel]);
                }
            } else if let Entry::Occupied(_e) = self.active_notes.entry(n.id) {
                self.active_notes.remove(&n.id);
                midi_out.send(&[0x80, n.note, n.vel]);
            }
        }

        notes_state
    }
    fn pause_resume(&mut self) {
        self.clear();
        self.timer.pause_resume();
    }
    fn set_time(&mut self, time: f32) {
        self.timer.set_time(time * 1000.0);
        self.clear();
    }
    fn clear(&mut self) {
        for (_id, n) in self.active_notes.iter() {
            self.midi_device.send(&[0x80, *n, 0]);
        }
        self.active_notes.clear();
    }
}

impl Drop for Player {
    fn drop(&mut self) {
        self.clear();
    }
}
