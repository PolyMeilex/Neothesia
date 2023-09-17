use crate::{config::Config, render::KeyboardRenderer};

pub fn user_midi_event(keyboard: &mut KeyboardRenderer, event: &crate::midi_event::MidiEvent) {
    use crate::midi_event::MidiEvent;

    let range_start = keyboard.range().start() as usize;

    let (is_on, key) = match event {
        MidiEvent::NoteOn { key, .. } => (true, key),
        MidiEvent::NoteOff { key, .. } => (false, key),
    };

    if keyboard.range().contains(*key) {
        let id = *key as usize - range_start;
        let key = &mut keyboard.key_states_mut()[id];

        key.set_pressed_by_user(is_on);
        keyboard.queue_reupload();
    }
}

pub fn file_midi_events(
    keyboard: &mut KeyboardRenderer,
    config: &Config,
    events: &[&midi_file::MidiEvent],
) {
    use midi_file::midly::MidiMessage;

    let range_start = keyboard.range().start() as usize;

    for e in events {
        let (is_on, key) = match e.message {
            MidiMessage::NoteOn { key, .. } => (true, key.as_int()),
            MidiMessage::NoteOff { key, .. } => (false, key.as_int()),
            _ => continue,
        };

        if keyboard.range().contains(key) && e.channel != 9 {
            let id = key as usize - range_start;
            let key = &mut keyboard.key_states_mut()[id];

            if is_on {
                let color = &config.color_schema[e.track_color_id % config.color_schema.len()];
                key.pressed_by_file_on(color);
            } else {
                key.pressed_by_file_off();
            }

            keyboard.queue_reupload();
        }
    }
}
