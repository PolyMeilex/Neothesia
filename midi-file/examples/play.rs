use std::{collections::BTreeMap, io::Write, time::Duration};

use midi_io::{MidiOutputConnection, MidiOutputManager, MidiOutputPort};
use midly::{MetaMessage, Smf, Timing, TrackEvent};

fn connect_midi_out() -> MidiOutputConnection {
    let manager = MidiOutputManager::new().unwrap();

    let mut out_ports = manager.outputs();
    let out_port: MidiOutputPort = match out_ports.len() {
        0 => panic!("MidiOut not found"),
        1 => {
            println!("Choosing the only available output port: {}", &out_ports[0]);
            out_ports.remove(0)
        }
        _ => {
            println!("\nAvailable output ports:");
            for (i, p) in out_ports.iter().enumerate() {
                println!("{i}: {p}");
            }
            print!("Please select output port: ");
            std::io::stdout().flush().unwrap();
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).unwrap();
            out_ports.remove(input.trim().parse::<usize>().unwrap())
        }
    };

    MidiOutputManager::connect_output(out_port).unwrap()
}

fn build_schedule(smf: Smf) -> BTreeMap<u64, Vec<TrackEvent>> {
    let mut schedule: BTreeMap<u64, Vec<TrackEvent>> = BTreeMap::new();
    for events in smf.tracks {
        let mut pulses = 0;
        for event in events {
            pulses += event.delta.as_int() as u64;
            schedule.entry(pulses).or_default().push(event);
        }
    }
    schedule
}

fn pulse_to_duration(pulses: u64, tempo: u32, pulses_per_quarter_note: u16) -> Duration {
    let u_time = pulses as f64 / pulses_per_quarter_note as f64;
    let time = (u_time * tempo as f64).floor() as u64;
    Duration::from_micros(time)
}

fn main() {
    let mut out_port = connect_midi_out();

    // let data = std::fs::read("/home/poly/Downloads/hopmon_package/Music01.mid").unwrap();
    // let data = std::fs::read("/home/poly/Downloads/mididownload.mid").unwrap();
    let data = std::fs::read("../test.mid").unwrap();
    let smf = Smf::parse(&data).unwrap();

    let pulses_per_quarter_note = match smf.header.timing {
        Timing::Metrical(t) => t.as_int(),
        Timing::Timecode(_fps, _u) => {
            panic!("Unsupported Timing::Timecode");
        }
    };

    let schedule = build_schedule(smf);

    let mut curr_tempo = 500_000;
    let mut last_pulses = 0;
    let mut buf = Vec::with_capacity(8);

    for (pulses, events) in schedule {
        std::thread::sleep(pulse_to_duration(
            pulses - last_pulses,
            curr_tempo,
            pulses_per_quarter_note,
        ));

        last_pulses = pulses;

        for event in events {
            if let midly::TrackEventKind::Meta(MetaMessage::Tempo(tempo)) = event.kind {
                curr_tempo = tempo.as_int();
            }

            if let Some(event) = event.kind.as_live_event() {
                buf.clear();
                event.write(&mut buf).unwrap();
                out_port.send(&buf).ok();
            }
        }
    }
}
