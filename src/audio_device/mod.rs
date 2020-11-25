use std::io::prelude::*;
use std::io::BufReader;
use std::io::Cursor;
use std::io::{Read, Seek};

pub struct Sinks {
    sinks: Vec<rodio::Sink>,
}

impl Sinks {
    fn new() -> Self {
        let sinks = Vec::new();

        Self { sinks }
    }

    fn play(&mut self, stream_handle: &rodio::OutputStreamHandle, buff: Vec<u8>) {
        let sink = rodio::Sink::try_new(stream_handle).unwrap();

        let decoder = rodio::Decoder::new(Cursor::new(buff)).unwrap();
        sink.append(decoder);

        self.sinks.push(sink);

        if self.sinks.len() > 20 {
            self.sinks.remove(0);
        }
    }
}

pub struct SoundManager {
    stream: rodio::OutputStream,
    stream_handle: rodio::OutputStreamHandle,
    sinks: Sinks,
    samples: Vec<(String, Vec<u8>)>,
}
impl SoundManager {
    pub fn new() -> Self {
        let (stream, stream_handle) = rodio::OutputStream::try_default().unwrap();

        let mut samples = Vec::new();

        for oct in 1..=7 {
            let dir = get_oct(oct);
            for f in dir {
                let path = format!("piano_samples/{}.mp3", f);
                let mut file = std::fs::File::open(&std::path::PathBuf::from(&path)).unwrap();

                let mut buff = Vec::new();
                file.read_to_end(&mut buff).unwrap();
                samples.push((path, buff));
            }
        }

        let sinks = Sinks::new();

        Self {
            stream,
            stream_handle,
            sinks,
            samples,
        }
    }

    pub fn play(&mut self, id: usize) {
        println!("{},{}", id, self.samples[id].0);
        self.sinks
            .play(&self.stream_handle, self.samples[id].1.clone());
    }
}

fn get_oct(id: i32) -> Vec<String> {
    vec![
        format!("C{}", id),
        format!("Db{}", id),
        format!("D{}", id),
        format!("Eb{}", id),
        format!("E{}", id),
        format!("F{}", id),
        format!("Gb{}", id),
        format!("G{}", id),
        format!("Ab{}", id),
        format!("A{}", id),
        format!("Bb{}", id),
        format!("B{}", id),
    ]
}
