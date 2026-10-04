pub mod menu {
    pub fn exit_question() -> String {
        "Do you want to exit?".into()
    }

    pub fn no() -> String {
        "No".into()
    }

    pub fn yes() -> String {
        "Yes".into()
    }

    pub fn loading() -> String {
        "Loading...".into()
    }

    pub fn select_file() -> String {
        "Select File".into()
    }

    pub fn settings() -> String {
        "Settings".into()
    }

    pub fn exit() -> String {
        "Exit".into()
    }

    pub fn freeplay() -> String {
        "FreePlay".into()
    }

    pub fn play() -> String {
        "Play".into()
    }

    pub fn tracks() -> String {
        "Tracks".into()
    }
}

pub mod freeplay {
    use std::path::Path;

    pub fn no_note_events_recorded() -> String {
        "No note events recorded".into()
    }

    pub fn failed_to_write_midi_file() -> String {
        "Failed to write MIDI file".into()
    }

    pub fn recording(seconds: f32) -> String {
        format!("Recording {seconds:.1}s")
    }

    pub fn recorded(seconds: f32) -> String {
        format!("Recorded {seconds:.1}s")
    }

    pub fn saved_recording_to(path: &Path) -> String {
        format!("Saved recording to {}", path.display())
    }
}

pub mod settings {
    pub fn output() -> String {
        "Output".into()
    }

    pub fn input() -> String {
        "Input".into()
    }

    pub fn note_range() -> String {
        "Note Range".into()
    }

    pub fn range_start() -> String {
        "Start".into()
    }

    pub fn range_end() -> String {
        "End".into()
    }

    pub fn render() -> String {
        "Render".into()
    }

    pub fn vertical_guidelines() -> String {
        "Vertical Guidelines".into()
    }

    pub fn vertical_guidelines_subtitle() -> String {
        "Display octave indicators".into()
    }

    pub fn horizontal_guidelines() -> String {
        "Horizontal Guidelines".into()
    }

    pub fn horizontal_guidelines_subtitle() -> String {
        "Display measure/bar indicators".into()
    }

    pub fn glow() -> String {
        "Glow".into()
    }

    pub fn glow_subtitle() -> String {
        "Key glow effect".into()
    }

    pub fn note_labels() -> String {
        "Note Labels".into()
    }

    pub fn note_labels_subtitle() -> String {
        "Display waterfall note labels".into()
    }

    pub fn soundfont() -> String {
        "SoundFont".into()
    }

    pub fn select_file() -> String {
        "Select File".into()
    }

    pub fn audio_gain() -> String {
        "Audio Gain".into()
    }

    pub fn separate_channels() -> String {
        "Separate Channels".into()
    }

    pub fn separate_channels_subtitle() -> String {
        "Assign different MIDI channel to each track".into()
    }

    pub fn auto_detect() -> String {
        "Auto-detect".into()
    }

    pub fn auto_detect_subtitle() -> String {
        "Auto-detect range from connected keyboard".into()
    }

    pub fn detect() -> String {
        "Detect".into()
    }

    pub fn detection_subtitle() -> String {
        "Play the far-left and far-right key on your keyboard...".into()
    }

    pub fn cancel() -> String {
        "Cancel".into()
    }

    pub fn detection_in_progress(step: usize, total: usize) -> String {
        format!("Detection in progress... Step {step}/{total}")
    }
}

pub mod playing {
    pub fn speed(speed: f32) -> String {
        format!("Speed: {speed}")
    }

    pub fn animation_speed(speed: f32) -> String {
        format!("Animation Speed: {speed}")
    }

    pub fn offset(offset: f32) -> String {
        format!("Offset: {offset}")
    }

    pub fn display() -> String {
        "Display".into()
    }

    pub fn chord_identifier() -> String {
        "Chord Identifier".into()
    }

    pub fn chord_identifier_subtitle() -> String {
        "Display chord above keyboard".into()
    }
}
