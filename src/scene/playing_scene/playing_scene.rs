use super::super::{Scene, SceneEvent, SceneType};
use super::keyboard::PianoKeyboard;
use super::notes::Notes;

use crate::ui::Ui;
use crate::wgpu_jumpstart::Gpu;
use crate::MainState;

use winit::event::VirtualKeyCode;

pub struct PlayingScene {
    piano_keyboard: PianoKeyboard,
    notes: Notes,
    midi: lib_midi::Midi,
    player: Player,
}

impl PlayingScene {
    pub fn new(
        gpu: &mut Gpu,
        state: &mut MainState,
        midi: lib_midi::Midi,
        device_id: usize,
    ) -> Self {
        let piano_keyboard = PianoKeyboard::new(state, &gpu);
        let notes = Notes::new(state, &gpu, &midi);

        state.time_menager.start_timer();

        Self {
            piano_keyboard,
            notes,
            midi,
            player: Player::new(device_id),
        }
    }
}

impl Scene for PlayingScene {
    fn state_type(&self) -> SceneType {
        SceneType::Playing
    }
    fn resize(&mut self, state: &mut MainState, gpu: &mut Gpu) {
        self.piano_keyboard.resize(state, gpu);
        self.notes
            .resize(state, gpu, &self.piano_keyboard.all_keys, &self.midi);
    }
    fn update(&mut self, state: &mut MainState, gpu: &mut Gpu, _ui: &mut Ui) -> SceneEvent {
        if let Some(time) = state.time_menager.timer_get_elapsed() {
            let time = time as f32 / 1000.0;

            let notes_on = self.player.update(&self.midi, time);
            self.piano_keyboard.update_notes(gpu, notes_on);
            self.notes.update(gpu, time);
        }

        SceneEvent::None
    }
    fn render(&mut self, state: &mut MainState, gpu: &mut Gpu, frame: &wgpu::SwapChainOutput) {
        self.notes.render(state, gpu, frame);
        self.piano_keyboard.render(state, gpu, frame);
    }
    fn key_released(&mut self, state: &mut MainState, key: VirtualKeyCode) {
        match key {
            VirtualKeyCode::Space => {
                state.time_menager.pause_resume_timer();
            }
            _ => {}
        }
    }
}

use std::collections::HashMap;
struct Player {
    midi_device: crate::midi_device::MidiDevicesMenager,
    active_notes: HashMap<usize, u8>,
}

impl Player {
    fn new(device_id: usize) -> Self {
        let mut midi_device = crate::midi_device::MidiDevicesMenager::new();

        log::info!("{:?}", midi_device.get_outs());

        midi_device.connect_out(device_id);
        Self {
            midi_device,
            active_notes: HashMap::new(),
        }
    }
    fn update(&mut self, midi: &lib_midi::Midi, time: f32) -> [bool; 88] {
        let midi_out = &mut self.midi_device;

        let mut notes_state: [bool; 88] = [false; 88];

        for n in midi
            .merged_track
            .notes
            .iter()
            .filter(|n| n.start <= time && n.start + n.duration + 0.5 > time)
        {
            use std::collections::hash_map::Entry;

            if n.start + n.duration >= time {
                if n.note >= 21 && n.note <= 108 {
                    notes_state[n.note as usize - 21] = true;
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
}

impl Drop for Player {
    fn drop(&mut self) {
        for (_id, n) in self.active_notes.iter() {
            self.midi_device.send(&[0x80, *n, 0]);
        }
    }
}
