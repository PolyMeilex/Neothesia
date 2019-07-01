extern crate lib_midi;
extern crate midir;
use midir::MidiOutput;


extern crate colored;
use colored::*;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    println!("Example Command: neothesia ~/my_midi_file.mid 1 (Id of midi output)");

    let midi: lib_midi::Midi;
    if args.len() > 1 {
        println!("Playing {}", args[1]);
        midi = lib_midi::read_file(&args[1]);
    } else {
        println!("Playing /tests/Simple Timeing.mid");
        println!("It should take 12.00s");
        println!("Gap between notes should be around 1s");
        midi = lib_midi::read_file("./tests/Simple Timeing.mid");
    }


    let midi_out = MidiOutput::new("midi").unwrap();

    println!("\nAvailable output ports:");
    for i in 0..midi_out.port_count() {
        println!("{}: {}", i, midi_out.port_name(i).unwrap());
    }

    let out_port: usize;

    if args.len() > 2 {
        out_port = args[2].parse::<usize>().unwrap();
    } else {
        out_port = 1;
    }

    println!("Using Port Number {}", out_port);

    let mut conn_out = midi_out.connect(out_port, "out").unwrap();

    let start_time = std::time::SystemTime::now();

    let mut index = 0;
    if midi.merged_track.notes.len() == 0 {
        panic!(
            "No Notes In Track For Some Reason \n {:?}",
            midi.merged_track
        )
    }
    let mut note = &midi.merged_track.notes[index];

    let mut notes_on: Vec<&lib_midi::track::MidiNote> = Vec::new();
    loop {
        let time = std::time::SystemTime::now()
            .duration_since(start_time)
            .unwrap()
            .as_millis() as f64
            / 1000.0;


        notes_on.retain(|no| {
            let delete = {
                if time >= no.start + no.duration {
                    conn_out.send(&[0x80, no.note, no.vel]).unwrap();
                    println!("{}{}", " ".repeat(no.note as usize), "#".red().bold());
                    true
                } else {
                    false
                }
            };
            !delete
        });


        if time >= note.start {
            // println!("{} \t T:{} \t {}", note.note, note.track_id, time);
            println!(
                "{}{}{:.2}",
                " ".repeat(note.note as usize),
                "#".green().bold(),
                note.start
            );
            if note.ch != 9 {
                conn_out.send(&[0x90, note.note, note.vel]).unwrap();
                notes_on.push(note);
            }

            index += 1;
            if index >= midi.merged_track.notes.len() {
                // TODO: Break After all notes are off in notes_on vec;
                // Temporary solution, stop all notes befor break
                for no in notes_on {
                    conn_out.send(&[0x80, no.note, no.vel]).unwrap();
                }
                break;
            }
            note = &midi.merged_track.notes[index];
        }
    }

}
