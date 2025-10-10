use clap::{Command, arg, value_parser};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Args {
    pub midi: PathBuf,
    pub out: PathBuf,
    pub soundfont: Option<PathBuf>,
    pub width: u32,
    pub height: u32,
}

impl Args {
    pub fn get() -> Self {
        let matches = Command::new("Neothesia")
            .about("MIDI visualization to video encoder")
            .arg(
                arg!([MIDI_FILE])
                    .required(true)
                    .value_parser(value_parser!(PathBuf)),
            )
            .arg(
                arg!([MP4_FILE])
                    .required(true)
                    .value_parser(value_parser!(PathBuf)),
            )
            .arg(
                arg!(--soundfont <SF2_FILE>)
                    .required(false)
                    .value_parser(value_parser!(PathBuf)),
            )
            .arg(arg!(--width <PIXELS>).required(false))
            .arg(arg!(--height <PIXELS>).required(false))
            .get_matches();

        let width = matches
            .get_one::<String>("width")
            .map(|w| w.parse::<u32>().unwrap())
            .unwrap_or(1920);
        let height = matches
            .get_one::<String>("height")
            .map(|w| w.parse::<u32>().unwrap())
            .unwrap_or(1080);

        if !width.is_multiple_of(2) || !height.is_multiple_of(2) {
            eprintln!("width and height must be a multiple of two");
            std::process::exit(1);
        }

        Self {
            midi: matches.get_one::<PathBuf>("MIDI_FILE").unwrap().clone(),
            out: matches.get_one::<PathBuf>("MP4_FILE").unwrap().clone(),
            soundfont: matches.get_one::<PathBuf>("soundfont").cloned(),
            width,
            height,
        }
    }
}
