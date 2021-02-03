use crate::{config::Config, output_manager::OutputManager};

pub struct MainState {
    pub midi_file: Option<lib_midi::Midi>,
    pub output_manager: OutputManager,

    pub config: Config,
}

impl MainState {
    pub fn new() -> Self {
        let args: Vec<String> = std::env::args().collect();

        let midi_file = if args.len() > 1 {
            if let Ok(midi) = lib_midi::Midi::new(&args[1]) {
                Some(midi)
            } else {
                None
            }
        } else {
            None
        };

        Self {
            midi_file,
            output_manager: OutputManager::new(),

            config: Config::new(),
        }
    }
}
