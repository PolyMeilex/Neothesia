use super::RewindController;
use crate::{target::Target, utils::timer::Timer, OutputManager};
use lib_midi::MidiNote;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::midi_event::MidiEvent;

pub struct MidiPlayer {
    midi_first_note_start: f32,
    midi_last_note_end: f32,
    active_notes: HashMap<usize, MidiNote>,
    timer: Timer,
    percentage: f32,
    time: f32,

    rewind_controller: RewindController,
    #[cfg(feature = "play_along")]
    play_along_controller: Option<PlayAlongController>,

    output_manager: Rc<RefCell<OutputManager>>,
}

impl MidiPlayer {
    pub fn new(target: &mut Target) -> Self {
        let midi_file = target.midi_file.as_ref().unwrap();

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
        let play_along_controller = if target.state.config.play_along {
            PlayAlongController::new()
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

            rewind_controller: RewindController::None,
            #[cfg(feature = "play_along")]
            play_along_controller,

            output_manager: target.output_manager.clone(),
        };
        player.update(target);

        player
    }

    /// When playing: returns midi events
    ///
    /// When paused: returns None
    pub fn update(&mut self, target: &mut Target) -> Option<Vec<MidiEvent>> {
        if let RewindController::Keyboard { speed, .. } = self.rewind_controller {
            let p = self.percentage + speed;
            self.set_percentage_time(target, p);
        }

        self.timer.update();
        let raw_time = self.timer.get_elapsed() / 1000.0 * target.config.speed_multiplier;
        self.percentage = raw_time / (self.midi_last_note_end + 3.0);
        self.time = raw_time + self.midi_first_note_start - 3.0;

        let mut events = Vec::new();

        #[cfg(feature = "play_along")]
        if let Some(controller) = &mut self.play_along_controller {
            controller.update(target, &mut events, &mut self.timer);
        }

        if self.timer.paused {
            return Some(events);
        };

        let filtered: Vec<&lib_midi::MidiNote> = target
            .midi_file
            .as_ref()
            .unwrap()
            .merged_track
            .notes
            .iter()
            .filter(|n| n.start <= self.time && n.start + n.duration + 0.5 > self.time)
            .collect();

        for n in filtered {
            use std::collections::hash_map::Entry;

            if n.start + n.duration >= self.time {
                if let Entry::Vacant(_e) = self.active_notes.entry(n.id) {
                    self.active_notes.insert(n.id, n.clone());

                    #[cfg(feature = "play_along")]
                    if let Some(controller) = &mut self.play_along_controller {
                        controller.require_note(&mut self.timer, &n);
                    }

                    events.push(MidiEvent::NoteOn {
                        channel: n.ch,
                        track_id: n.track_id,
                        key: n.note,
                        vel: n.vel,
                    });
                }
            } else if let Entry::Occupied(_e) = self.active_notes.entry(n.id) {
                self.active_notes.remove(&n.id);

                events.push(MidiEvent::NoteOff {
                    channel: n.ch,
                    key: n.note,
                });
            }
        }

        for event in events.iter() {
            self.output_manager.borrow_mut().midi_event((*event).into());
        }

        Some(events)
    }

    pub fn clear(&mut self) {
        for (_, n) in self.active_notes.iter() {
            self.output_manager.borrow_mut().midi_event(
                MidiEvent::NoteOff {
                    channel: n.ch,
                    key: n.note,
                }
                .into(),
            )
        }

        self.active_notes.clear();

        #[cfg(feature = "play_along")]
        if let Some(controller) = &mut self.play_along_controller {
            controller.clear();
        }
    }
}

impl MidiPlayer {
    pub fn start(&mut self) {
        self.timer.start();
    }

    pub fn pause_resume(&mut self) {
        self.clear();
        self.timer.pause_resume();
    }

    pub fn start_rewind(&mut self, controller: RewindController) {
        self.timer.pause();
        self.rewind_controller = controller;
    }

    pub fn stop_rewind(&mut self) {
        let controller = std::mem::replace(&mut self.rewind_controller, RewindController::None);

        let was_paused = match controller {
            RewindController::Keyboard { was_paused, .. } => was_paused,
            RewindController::Mouse { was_paused } => was_paused,
            RewindController::None => return,
        };

        if !was_paused {
            self.timer.resume();
        }
    }

    pub fn set_time(&mut self, time: f32) {
        self.timer.set_time(time * 1000.0);
        self.clear();
    }

    pub fn set_percentage_time(&mut self, target: &mut Target, p: f32) {
        self.set_time(p * (self.midi_last_note_end + 3.0) / target.config.speed_multiplier);
    }

    pub fn percentage(&self) -> f32 {
        self.percentage
    }

    pub fn time(&self) -> f32 {
        self.time
    }

    pub fn rewind_controller(&self) -> &RewindController {
        &self.rewind_controller
    }

    pub fn is_rewinding(&self) -> bool {
        self.rewind_controller.is_rewinding()
    }

    pub fn is_paused(&self) -> bool {
        self.timer.paused
    }
}

impl Drop for MidiPlayer {
    fn drop(&mut self) {
        self.clear();
    }
}

#[cfg(feature = "play_along")]
use std::sync::{mpsc, Arc, Mutex};

#[cfg(feature = "play_along")]
struct PlayAlongController {
    _midi_in_conn: midir::MidiInputConnection<()>,
    midi_in_rec: mpsc::Receiver<(bool, u8, u8)>,

    input_pressed_keys: [bool; 88],
    required_notes: Arc<Mutex<HashMap<u8, MidiNote>>>,
    waiting_for_note: bool,
}

#[cfg(feature = "play_along")]
impl PlayAlongController {
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

    fn update(&mut self, target: &mut Target, notes_state: &mut Vec<MidiEvent>, timer: &mut Timer) {
        if let Ok(event) = self.midi_in_rec.try_recv() {
            if event.0 {
                self.input_pressed_keys[event.1 as usize - 21] = true;
                target
                    .output_manager
                    .borrow_mut()
                    .midi_event(midi::Message::NoteOn(midi::Ch1, event.1, event.2));

                notes_state.push(MidiEvent::NoteOn {
                    channel: 0,
                    track_id: 0,
                    key: event.1,
                    vel: event.2,
                });
            } else {
                self.input_pressed_keys[event.1 as usize - 21] = false;
                target
                    .output_manager
                    .borrow_mut()
                    .midi_event(midi::Message::NoteOff(midi::Ch1, event.1, event.2));

                notes_state.push(MidiEvent::NoteOff {
                    channel: 0,
                    key: event.1,
                });
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
