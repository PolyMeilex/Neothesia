use std::{error::Error, path::Path, sync::mpsc::Receiver};

use crate::output_manager::{OutputConnection, OutputDescriptor};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

#[cfg(all(feature = "fluid-synth", not(feature = "oxi-synth")))]
const SAMPLES_SIZE: usize = 1410;

pub struct SynthBackend {
    _host: cpal::Host,
    device: cpal::Device,

    stream_config: cpal::StreamConfig,
    sample_format: cpal::SampleFormat,
}

impl SynthBackend {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let host = cpal::default_host();

        let device = host
            .default_output_device()
            .ok_or("failed to find a default output device")?;

        let config = device.default_output_config()?;
        let sample_format = config.sample_format();

        let stream_config: cpal::StreamConfig = config.into();

        Ok(Self {
            _host: host,
            device,

            stream_config,
            sample_format,
        })
    }

    fn run<T: cpal::SizedSample + cpal::FromSample<f32>>(
        &self,
        rx: Receiver<oxisynth::MidiEvent>,
        path: &Path,
    ) -> cpal::Stream {
        #[cfg(all(feature = "fluid-synth", not(feature = "oxi-synth")))]
        let mut next_value = {
            use fluidlite::{IsSettings, Settings};

            let synth = {
                let sample_rate = self.stream_config.sample_rate.0;

                let settings = Settings::new().unwrap();

                let rate = settings.pick::<_, f64>("synth.sample-rate").unwrap();
                rate.set(sample_rate as f64);

                let synth = fluidlite::Synth::new(settings).unwrap();
                synth.sfload(path, true).unwrap();
                synth.set_gain(1.0);

                synth
            };

            let mut sample_clock = 0;
            let mut buff: [f32; SAMPLES_SIZE] = [0.0f32; SAMPLES_SIZE];

            move || {
                let l = buff[sample_clock];
                let r = buff[sample_clock + 1];

                sample_clock += 2;

                if sample_clock == SAMPLES_SIZE {
                    synth.write(buff.as_mut()).unwrap();
                    sample_clock = 0;
                }

                if let Ok(e) = rx.try_recv() {
                    match e {
                        oxisynth::MidiEvent::NoteOn { channel, key, vel } => {
                            synth.note_on(channel as u32, key as u32, vel as u32).ok();
                        }
                        oxisynth::MidiEvent::NoteOff { channel, key } => {
                            synth.note_off(channel as u32, key as u32).ok();
                        }
                    }
                }

                (l, r)
            }
        };

        #[cfg(all(feature = "oxi-synth", not(feature = "fluid-synth")))]
        let mut next_value = {
            let sample_rate = self.stream_config.sample_rate.0 as f32;

            let mut synth = oxisynth::Synth::new(oxisynth::SynthDescriptor {
                sample_rate,
                gain: 1.0,
                ..Default::default()
            })
            .unwrap();

            {
                let mut file = std::fs::File::open(path).unwrap();
                let font = oxisynth::SoundFont::load(&mut file).unwrap();
                synth.add_font(font, true);
            }

            move || {
                let (l, r) = synth.read_next();

                if let Ok(event) = rx.try_recv() {
                    synth.send_event(event).ok();
                }

                (l, r)
            }
        };

        let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

        let channels = self.stream_config.channels as usize;

        let stream = self
            .device
            .build_output_stream(
                &self.stream_config,
                move |output: &mut [T], _: &cpal::OutputCallbackInfo| {
                    for frame in output.chunks_mut(channels) {
                        let (l, r) = next_value();

                        let l = T::from_sample(l);
                        let r = T::from_sample(r);

                        let channels = [l, r];

                        for (id, sample) in frame.iter_mut().enumerate() {
                            *sample = channels[id % 2];
                        }
                    }
                },
                err_fn,
                None,
            )
            .unwrap();
        stream.play().unwrap();

        stream
    }

    pub fn new_output_connection(&mut self, path: &Path) -> SynthOutputConnection {
        let (tx, rx) = std::sync::mpsc::channel::<oxisynth::MidiEvent>();
        let _stream = match self.sample_format {
            cpal::SampleFormat::I8 => self.run::<i8>(rx, path),
            cpal::SampleFormat::I16 => self.run::<i16>(rx, path),
            cpal::SampleFormat::I32 => self.run::<i32>(rx, path),
            cpal::SampleFormat::I64 => self.run::<i64>(rx, path),

            cpal::SampleFormat::U8 => self.run::<u8>(rx, path),
            cpal::SampleFormat::U16 => self.run::<u16>(rx, path),
            cpal::SampleFormat::U32 => self.run::<u32>(rx, path),
            cpal::SampleFormat::U64 => self.run::<u64>(rx, path),

            cpal::SampleFormat::F32 => self.run::<f32>(rx, path),
            cpal::SampleFormat::F64 => self.run::<f64>(rx, path),
            sample_format => unimplemented!("Unsupported sample format '{sample_format}'"),
        };

        SynthOutputConnection { _stream, tx }
    }

    pub fn get_outputs(&self) -> Vec<OutputDescriptor> {
        vec![OutputDescriptor::Synth(None)]
    }
}

pub struct SynthOutputConnection {
    _stream: cpal::Stream,
    tx: std::sync::mpsc::Sender<oxisynth::MidiEvent>,
}

impl OutputConnection for SynthOutputConnection {
    fn midi_event(&mut self, msg: &lib_midi::MidiEvent) {
        let event = libmidi_to_oxisynth_event(msg);
        self.tx.send(event).ok();
    }

    fn stop_all(&mut self) {
        self.tx.send(oxisynth::MidiEvent::SystemReset).ok();
    }
}

fn libmidi_to_oxisynth_event(msg: &lib_midi::MidiEvent) -> oxisynth::MidiEvent {
    use lib_midi::midly;

    let channel = msg.channel;
    match msg.message {
        midly::MidiMessage::NoteOff { key, .. } => oxisynth::MidiEvent::NoteOff {
            channel,
            key: key.as_int(),
        },
        midly::MidiMessage::NoteOn { key, vel } => oxisynth::MidiEvent::NoteOn {
            channel,
            key: key.as_int(),
            vel: vel.as_int(),
        },
        midly::MidiMessage::Aftertouch { key, vel } => oxisynth::MidiEvent::PolyphonicKeyPressure {
            channel,
            key: key.as_int(),
            value: vel.as_int(),
        },
        midly::MidiMessage::Controller { controller, value } => {
            oxisynth::MidiEvent::ControlChange {
                channel,
                ctrl: controller.as_int(),
                value: value.as_int(),
            }
        }
        midly::MidiMessage::ProgramChange { program } => oxisynth::MidiEvent::ProgramChange {
            channel,
            program_id: program.as_int(),
        },
        midly::MidiMessage::ChannelAftertouch { vel } => oxisynth::MidiEvent::ChannelPressure {
            channel,
            value: vel.as_int(),
        },
        midly::MidiMessage::PitchBend { bend } => oxisynth::MidiEvent::PitchBend {
            channel,
            value: bend.0.as_int(),
        },
    }
}
