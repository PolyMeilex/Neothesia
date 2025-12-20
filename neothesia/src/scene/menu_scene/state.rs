use std::collections::VecDeque;

use crate::{NeothesiaEvent, context::Context, output_manager::OutputDescriptor, song::Song};

type InputDescriptor = midi_io::MidiInputPort;

pub struct UiState {
    pub outputs: Vec<OutputDescriptor>,
    pub selected_output: Option<OutputDescriptor>,

    pub inputs: Vec<InputDescriptor>,
    pub selected_input: Option<InputDescriptor>,

    pub is_loading: bool,

    pub song: Option<Song>,

    page_stack: VecDeque<Page>,
}

impl UiState {
    pub fn new(_ctx: &Context, song: Option<Song>) -> Self {
        let mut page_stack = VecDeque::new();
        page_stack.push_front(Page::Main);

        Self {
            outputs: Vec::new(),
            selected_output: None,
            inputs: Vec::new(),
            selected_input: None,
            is_loading: false,
            song,

            page_stack,
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
                self.selected_input = Some(input.clone());
            } else {
                self.selected_input = self.inputs.first().cloned();
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
        ctx.input_manager.connect_input(port);
    }
}

pub fn play(data: &UiState, ctx: &mut Context) {
    let Some(song) = data.song.as_ref() else {
        return;
    };

    connect_io(data, ctx);

    ctx.proxy
        .send_event(NeothesiaEvent::Play(song.clone()))
        .ok();
}

pub fn freeplay(data: &UiState, ctx: &mut Context) {
    connect_io(data, ctx);

    ctx.proxy
        .send_event(NeothesiaEvent::FreePlay(data.song.clone()))
        .ok();
}
