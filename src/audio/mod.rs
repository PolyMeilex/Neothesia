extern crate fluidlite_lib;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use fluidlite::{IsSettings, Settings};

const SAMPLES_SIZE: usize = 1410;

enum MidiEvent {
    NoteOn { ch: u8, key: u8, vel: u8 },
    NoteOff { ch: u8, key: u8 },
}

pub struct Synth {
    _host: cpal::Host,
    device: cpal::Device,
    stream_config: cpal::StreamConfig,
    stream: Option<cpal::Stream>,

    tx: std::sync::mpsc::Sender<MidiEvent>,
}
impl Synth {
    pub fn new() -> Self {
        let host = cpal::default_host();

        let device = host
            .default_output_device()
            .expect("failed to find a default output device");

        let config = device.default_output_config().unwrap();
        let sample_format = config.sample_format();

        let mut stream_config: cpal::StreamConfig = config.into();
        stream_config.sample_rate.0 = 44100;

        let (tx, rx) = std::sync::mpsc::channel::<MidiEvent>();

        let mut out = Synth {
            _host: host,
            device,
            stream_config,
            stream: None,

            tx,
        };

        match sample_format {
            cpal::SampleFormat::F32 => out.run::<f32>(rx),
            cpal::SampleFormat::I16 => out.run::<i16>(rx),
            cpal::SampleFormat::U16 => out.run::<u16>(rx),
        }

        out
    }

    fn run<T: cpal::Sample>(&mut self, rx: std::sync::mpsc::Receiver<MidiEvent>) {
        let mut buff: [f32; SAMPLES_SIZE] = [0.0f32; SAMPLES_SIZE];

        let synth = {
            let sample_rate = self.stream_config.sample_rate.0;

            let settings = Settings::new().unwrap();

            let rate = settings.pick::<_, f64>("synth.sample-rate").unwrap();
            rate.set((sample_rate / 2) as f64);

            println!("{:?},{:?}", rate.get(), sample_rate);

            let synth = fluidlite::Synth::new(settings).unwrap();
            synth.sfload("font.sf2", true).unwrap();
            synth.set_sample_rate(sample_rate as f32);
            synth.set_gain(1.0);

            synth
        };

        let mut sample_clock = 0;

        let mut next_value = move || {
            let out = buff[sample_clock];

            sample_clock += 1;

            if sample_clock == SAMPLES_SIZE {
                synth.write(buff.as_mut()).unwrap();
                sample_clock = 0;
            }

            if let Ok(e) = rx.try_recv() {
                match e {
                    MidiEvent::NoteOn { ch, key, vel } => {
                        synth.note_on(ch as u32, key as u32, vel as u32).ok();
                    }
                    MidiEvent::NoteOff { ch, key } => {
                        synth.note_off(ch as u32, key as u32).ok();
                    }
                }
            }

            out
        };

        let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

        let channels = self.stream_config.channels as usize;

        let stream = self
            .device
            .build_output_stream(
                &self.stream_config,
                move |output: &mut [T], _: &cpal::OutputCallbackInfo| {
                    for frame in output.chunks_mut(channels) {
                        let value: T = cpal::Sample::from::<f32>(&next_value());
                        for sample in frame.iter_mut() {
                            *sample = value;
                        }
                    }
                },
                err_fn,
            )
            .unwrap();
        stream.play().unwrap();

        self.stream = Some(stream);
    }

    pub fn note_on(&mut self, ch: u8, key: u8, vel: u8) {
        self.tx.send(MidiEvent::NoteOn { ch, key, vel }).ok();
    }

    pub fn note_off(&mut self, ch: u8, key: u8) {
        self.tx.send(MidiEvent::NoteOff { ch, key }).ok();
    }
}
