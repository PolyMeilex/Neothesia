use std::collections::VecDeque;
use std::path::PathBuf;

use crate::{NeothesiaEvent, context::Context, output_manager::OutputDescriptor, song::Song};

type InputDescriptor = midi_io::MidiInputPort;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Default)]
pub enum PlayMode {
    #[default]
    Watch,
    Learn,
    Play,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Default)]
pub enum HandSelection {
    #[default]
    Both,
    Left,
    Right,
}

pub struct UiState {
    pub outputs: Vec<OutputDescriptor>,
    pub selected_output: Option<OutputDescriptor>,

    pub inputs: Vec<InputDescriptor>,
    pub selected_input: Option<InputDescriptor>,

    pub is_loading: bool,

    pub song: Option<Song>,

    page_stack: VecDeque<Page>,

    // SoundFont folder management
    pub soundfont_folders: Vec<PathBuf>,
    pub discovered_soundfonts: Vec<crate::output_manager::SoundFontEntry>,
    pub current_soundfont_index: Option<usize>,

    // Play mode configuration
    pub play_mode: PlayMode,
    pub hand_selection: HandSelection,
}

impl UiState {
    pub fn new(ctx: &Context, song: Option<Song>) -> Self {
        let mut page_stack = VecDeque::new();
        page_stack.push_front(Page::Main);

        let soundfont_folders = ctx.config.synth_config.soundfont_folders().clone();
        let current_soundfont_index = ctx.config.synth_config.soundfont_index();

        // Discover SoundFonts from all folders
        let discovered_soundfonts = crate::output_manager::discover_soundfonts(&soundfont_folders);

        Self {
            outputs: Vec::new(),
            selected_output: None,
            inputs: Vec::new(),
            selected_input: None,
            is_loading: false,
            song,

            page_stack,
            soundfont_folders,
            discovered_soundfonts,
            current_soundfont_index,
            play_mode: PlayMode::default(),
            hand_selection: HandSelection::default(),
        }
    }

    pub fn song(&self) -> Option<&Song> {
        self.song.as_ref()
    }

    pub fn is_loading(&self) -> bool {
        self.is_loading
    }

    pub fn current(&self) -> &Page {
        self.page_stack.front().unwrap()
    }

    pub fn go_to(&mut self, page: Page) {
        self.page_stack.push_front(page);
    }

    pub fn go_back(&mut self) {
        match self.page_stack.len() {
            1 => {
                // Last page in the stack, let's go to exit page
                self.page_stack.push_front(Page::Exit);
            }
            _ => {
                self.page_stack.pop_front();
            }
        }
    }
}

impl UiState {
    pub fn tick(&mut self, ctx: &mut Context) {
        self.outputs = ctx.output_manager.outputs();
        self.inputs = ctx.input_manager.inputs();

        if self.selected_output.is_none() {
            if let Some(name) = ctx.config.output() {
                if let Some(out) = self
                    .outputs
                    .iter()
                    .find(|output| output.to_string().as_str() == name)
                {
                    self.selected_output = Some(out.clone());
                } else {
                    self.selected_output = self.outputs.first().cloned();
                }
            } else {
                self.selected_output = Some(OutputDescriptor::DummyOutput);
            }
        }

        if self.selected_input.is_none() {
            if let Some(input) = self
                .inputs
                .iter()
                .find(|input| Some(input.to_string().as_str()) == ctx.config.input())
            {
                log::info!("Auto-selecting MIDI input: '{}'", input.to_string());
                self.selected_input = Some(input.clone());
                // Immediately connect LUMI SysEx output when input is selected
                let port_name = input.to_string();
                log::info!("Connecting LUMI SysEx for input port: '{}'", port_name);
                ctx.output_manager.connect_lumi_by_port_name(&port_name);
            } else {
                if let Some(first) = self.inputs.first() {
                    log::info!("Selecting first available MIDI input: '{}'", first.to_string());
                    self.selected_input = Some(first.clone());
                    // Immediately connect LUMI SysEx output for first available input
                    let port_name = first.to_string();
                    log::info!("Connecting LUMI SysEx for input port: '{}'", port_name);
                    ctx.output_manager.connect_lumi_by_port_name(&port_name);
                }
            }
}
}
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Page {
    Exit,
    Main,
    Settings,
    TrackSelection,
    PlayMode,
}

fn connect_io(data: &UiState, ctx: &mut Context) {
    if let Some(out) = data.selected_output.clone() {
        let out = match out {
            #[cfg(feature = "synth")]
            OutputDescriptor::Synth(_) => {
                OutputDescriptor::Synth(ctx.config.soundfont_path().cloned())
            }
            o => o,
        };

        ctx.output_manager.connect(out);
        ctx.output_manager
            .connection()
            .set_gain(ctx.config.audio_gain());
    }

    if let Some(port) = data.selected_input.clone() {
        // LUMI SysEx output is already connected when input was selected in tick()
        // Just connect the regular input for MIDI note events
        ctx.input_manager.connect_input(port);
    }
}

pub fn play(data: &UiState, ctx: &mut Context) {
    play_with_config(data, ctx, PlayMode::Watch, HandSelection::Both);
}

pub fn play_with_config(data: &UiState, ctx: &mut Context, play_mode: PlayMode, hand_selection: HandSelection) {
    let Some(song) = data.song.as_ref() else {
        return;
    };

    let mut song = song.clone();
    
    song.config.wait_mode = play_mode == PlayMode::Learn;
    
    for (track_id, track_config) in song.config.tracks.iter_mut().enumerate() {
        if let Some(track) = song.file.tracks.get(track_id) {
            let is_drums = track.has_drums && !track.has_other_than_drums;
            
            if is_drums {
                track_config.visible = true;
                continue;
            }
            
            if hand_selection == HandSelection::Both {
                track_config.visible = true;
                for channel_config in track_config.channels.iter_mut() {
                    channel_config.active = true;
                }
                continue;
            }
            
            let mut channel_notes: std::collections::HashMap<u8, Vec<u8>> = std::collections::HashMap::new();
            for note in track.notes.as_ref() {
                channel_notes.entry(note.channel).or_default().push(note.note);
            }
            
            for channel_config in track_config.channels.iter_mut() {
                let notes = channel_notes.get(&channel_config.channel);
                
                // Only activate channels that have notes in this track
                let has_notes = notes.map(|n| !n.is_empty()).unwrap_or(false);
                
                if !has_notes {
                    // Channel has no notes - deactivate it
                    channel_config.active = false;
                    continue;
                }
                
                let avg_note = notes.and_then(|n| {
                    if n.is_empty() { None } else {
                        Some(n.iter().map(|&m| m as f32).sum::<f32>() / n.len() as f32)
                    }
                }).unwrap_or(60.0);
                
                match hand_selection {
                    HandSelection::Left => {
                        channel_config.active = avg_note < 60.0;
                    }
                    HandSelection::Right => {
                        channel_config.active = avg_note >= 60.0;
                    }
                    HandSelection::Both => {
                        channel_config.active = true;
                    }
                }
            }
            
            // Track is visible only if it has at least one active channel
            track_config.visible = track_config.channels.iter().any(|c| c.active);
            // Note: If filtering results in no active channels, that's the intended behavior
            // The user can adjust their hand selection if needed
        }
    }

    connect_io(data, ctx);

    ctx.proxy
        .send_event(NeothesiaEvent::Play(song))
        .ok();
}

pub fn freeplay(data: &UiState, ctx: &mut Context) {
    connect_io(data, ctx);

    ctx.proxy
        .send_event(NeothesiaEvent::FreePlay(data.song.clone()))
        .ok();
}
