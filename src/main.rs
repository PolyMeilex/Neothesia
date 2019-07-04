extern crate lib_midi;
extern crate midir;
use midir::MidiOutput;


extern crate colored;
use colored::*;


mod render;

#[macro_use]
extern crate glium;

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

    if midi.merged_track.notes.len() == 0 {
        panic!(
            "No Notes In Track For Some Reason \n {:?}",
            midi.merged_track
        )
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

    //
    // Render
    //

    use glium::glutin;

    let mut events_loop = glutin::EventsLoop::new();
    let wb = glutin::WindowBuilder::new()
        .with_title("Neothesia!")
        .with_dimensions(glutin::dpi::LogicalSize::new(1280.0, 720.0));

    let cb = glutin::ContextBuilder::new();
    let display = glium::Display::new(wb, cb, &events_loop).unwrap();

    let mut game_renderer = render::GameRenderer::new(&display);

    let notes = midi.merged_track.notes.clone();
    game_renderer.load_song(midi.merged_track);

    let start_time = std::time::Instant::now();
    let mut closed = false;

    std::thread::spawn(move || {
        let mut last_time = 0;
        let mut delta_time;
        let mut time_elapsed: u128 = 0;

        let mut index = 0;
        let mut note = &notes[index];
        let mut notes_on: Vec<&lib_midi::track::MidiNote> = Vec::new();

        while !closed {
            let current_time = start_time.elapsed().as_millis();
            delta_time = current_time - last_time;
            last_time = current_time;

            time_elapsed += delta_time;

            notes_on.retain(|no| {
                let delete = {
                    if time_elapsed as f64 / 1000.0 >= no.start + no.duration {
                        conn_out.send(&[0x80, no.note, no.vel]).unwrap();
                        true
                    } else {
                        false
                    }
                };
                !delete
            });


            if time_elapsed as f32 / 1000.0 >= note.start as f32 {
                if note.ch != 9 {
                    conn_out.send(&[0x90, note.note, note.vel]); //.unwrap();
                    notes_on.push(note);
                }

                index += 1;
                if index >= notes.len() {
                    // TODO: Break After all notes are off in notes_on vec;
                    // Temporary solution, stop all notes befor break
                    for no in notes_on.iter() {
                        conn_out.send(&[0x80, no.note, no.vel]); //.unwrap();
                    }
                    break;
                }
                note = &notes[index];
            }

        }
        return true;
    });


    let mut fps = 0.0;
    let mut last_time_fps = 0;

    //Delta Time
    let mut last_time = 0;
    let mut delta_time;

    let mut time_elapsed: u128 = 0;

    while !closed {
        let current_time = start_time.elapsed().as_millis();
        delta_time = current_time - last_time;
        last_time = current_time;
        time_elapsed += delta_time;

        // FPS
        fps += 1.0;
        if time_elapsed - last_time_fps > 1000 {
            last_time_fps = start_time.elapsed().as_millis();
            println!("{}", fps);
            fps = 0.0;
        }

        game_renderer.draw(time_elapsed);

        events_loop.poll_events(|ev| match ev {
            glutin::Event::WindowEvent { event, .. } => match event {
                glutin::WindowEvent::Resized(_size) => {
                    // We can't use glutin size becouse of winit problem
                    // Probably this one
                    // https://github.com/rust-windowing/glutin/issues/1087

                    // So we request size updated by glium
                    // It will be updated on next frame
                    game_renderer.update_size = true;
                }
                glutin::WindowEvent::CursorMoved { position, .. } => {
                    let pox_x = position.x;
                    let pox_y = position.y - game_renderer.viewport.bottom as f64;

                    let pox_x = pox_x / (game_renderer.viewport.width as f64 / 2.0) - 1.0;
                    let pox_y = -(pox_y / (game_renderer.viewport.height as f64 / 2.0) - 1.0);

                    game_renderer.m_pos_x = pox_x;
                    game_renderer.m_pos_y = pox_y;
                }
                glutin::WindowEvent::CloseRequested => closed = true,
                _ => (),
            },
            _ => (),
        });
    }
}
