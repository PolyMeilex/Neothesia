extern crate lib_midi;
extern crate midir;
use midir::{Ignore, MidiInput, MidiOutput};


extern crate colored;
use colored::*;

fn main() {
    let midi =
        lib_midi::read_file("/home/poly/Documents/Midi/Billy Joel - We Didn`t Start The Fire.mid");
    // let midi = lib_midi::read_file("/home/poly/timeing.mid");


    let midi_out = MidiOutput::new("midi").unwrap();
    let out_port = 1;
    let mut conn_out = midi_out.connect(out_port, "out").unwrap();

    let start_time = std::time::SystemTime::now();

    let mut index = 0;
    let mut note = &midi.merged_track.notes[index];

    let mut notes_on: Vec<&lib_midi::event_parser::MidiNote> = Vec::new();
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
                // TODO: BRAKE After all notes are off in notes_on vec;
                break;
            }
            note = &midi.merged_track.notes[index];
        }
    }

}
