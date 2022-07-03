use crate::{midi_event::MidiEvent, NeothesiaEvent};

pub struct InputManager {
    input: midi_io::MidiInputManager,
    tx: winit::event_loop::EventLoopProxy<NeothesiaEvent>,
    current_connection: Option<midi_io::MidiInputConnection>,
}

impl InputManager {
    pub fn new(tx: winit::event_loop::EventLoopProxy<NeothesiaEvent>) -> Self {
        let input = midi_io::MidiInputManager::new().unwrap();
        Self {
            input,
            tx,
            current_connection: None,
        }
    }

    pub fn inputs(&self) -> Vec<midi_io::MidiInputPort> {
        self.input.inputs()
    }

    pub fn connect_input(&mut self, port: midi_io::MidiInputPort) {
        let tx = self.tx.clone();
        self.current_connection = midi_io::MidiInputManager::connect_input(port, move |message| {
            if message.len() == 3 {
                if message[0] >= 0x90 && message[0] <= 0x9F {
                    let (s, ch) = midi::utils::from_status_byte(message[0]);
                    assert_eq!(s, 9);

                    let key = message[1];
                    let vel = message[2];

                    // Some keyboards send NoteOn event with vel 0 instead of NoteOff
                    if vel == 0 {
                        tx.send_event(NeothesiaEvent::MidiInput(MidiEvent::NoteOff {
                            channel: ch as u8,
                            key,
                        }))
                        .unwrap();
                    } else {
                        tx.send_event(NeothesiaEvent::MidiInput(MidiEvent::NoteOn {
                            channel: ch as u8,
                            track_id: 0,
                            key,
                            vel,
                        }))
                        .unwrap();
                    }
                } else if message[0] >= 0x80 && message[0] <= 0x8F {
                    let (s, ch) = midi::utils::from_status_byte(message[0]);
                    assert_eq!(s, 8);

                    tx.send_event(NeothesiaEvent::MidiInput(MidiEvent::NoteOff {
                        channel: ch as u8,
                        key: message[1],
                    }))
                    .unwrap();
                }
            }
        });
    }
}
