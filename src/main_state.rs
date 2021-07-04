use crate::config::Config;

pub struct MainState {
    pub midi_file: Option<lib_midi::Midi>,

    pub config: Config,
}

impl Default for MainState {
    fn default() -> Self {
        Self::new()
    }
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

            config: Config::new(),
        }
    }
}
