use crate::{config::Config, keyboard_renderer::KeyboardRenderer};

pub use crate::keyboard_renderer::KeyState;

pub fn user_midi_event(keyboard: &mut KeyboardRenderer, event: &crate::MidiEvent) {
    use crate::MidiEvent;

    let (is_on, key) = match event {
        MidiEvent::NoteOn { key, .. } => (true, key),
        MidiEvent::NoteOff { key, .. } => (false, key),
    };

    if keyboard.range().contains(*key) {
        let id = *key as usize - 21;
        let key = &mut keyboard.key_states_mut()[id];

        key.set_pressed_by_user(is_on);
        keyboard.queue_reupload();
    }
}

pub fn file_midi_events(
    keyboard: &mut KeyboardRenderer,
    config: &Config,
    events: &[lib_midi::MidiEvent],
) {
    use lib_midi::midly::MidiMessage;

    for e in events {
        let (is_on, key) = match e.message {
            MidiMessage::NoteOn { key, .. } => (true, key.as_int()),
            MidiMessage::NoteOff { key, .. } => (false, key.as_int()),
            _ => continue,
        };

        if keyboard.range().contains(key) && e.channel != 9 {
            let id = key as usize - 21;
            let key = &mut keyboard.key_states_mut()[id];

            if is_on {
                let color = &config.color_schema[e.track_id % config.color_schema.len()];
                key.pressed_by_file_on(color);
            } else {
                key.pressed_by_file_off();
            }

            keyboard.queue_reupload();
        }
    }
}
