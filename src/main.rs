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

    use glium::{glutin, Surface};

    let mut events_loop = glutin::EventsLoop::new();
    let wb = glutin::WindowBuilder::new();
    let cb = glutin::ContextBuilder::new();
    let display = glium::Display::new(wb, cb, &events_loop).unwrap();

    let mut game_renderer = render::GameRenderer::new(&display);


    // let mut index = 0;
    // if midi.merged_track.notes.len() == 0 {
    //     panic!(
    //         "No Notes In Track For Some Reason \n {:?}",
    //         midi.merged_track
    //     )
    // }
    // let mut note = &midi.merged_track.notes[index];
    // let mut notes_on: Vec<&lib_midi::track::MidiNote> = Vec::new();

    // #[derive(Copy, Clone)]
    // struct Vertex {
    //     pos: [f32; 2],
    // }
    // implement_vertex!(Vertex, pos);

    // let vertex1 = Vertex { pos: [-0.5, -0.5] };
    // let vertex2 = Vertex { pos: [0.5, -0.5] };
    // let vertex3 = Vertex { pos: [0.5, 0.5] };
    // let vertex4 = Vertex { pos: [-0.5, 0.5] };

    // let shape: [Vertex; 4] = [vertex1, vertex2, vertex3, vertex4];
    // let indices_vec: [u16; 6] = [0, 1, 3, 3, 1, 2];

    // let vertex_buffer = glium::VertexBuffer::new(&display, &shape).unwrap();
    // let indices = glium::IndexBuffer::new(
    //     &display,
    //     glium::index::PrimitiveType::TrianglesList,
    //     &indices_vec,
    // )
    // .unwrap();
    // let indices = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);

    // let vertex_shader_src = r#"
    //         #version 330

    //         in vec2 pos;

    //         uniform float t;

    //         #define notesCount 52.0

    //         out INTERFACE {
    //             vec2 uv;
    //             vec2 noteSize;
    //         } Out;


    //         void main() {
    //             Out.noteSize = vec2(0.9*2.0/notesCount, 1.0);

    //             const float a = (1.0/(notesCount-1.0)) * (2.0 - 2.0/notesCount);
    //             const float b = -1.0 + 1.0/notesCount;
    //             float noteShift = t * a + b;

    //             Out.uv = Out.noteSize * pos;

    //             gl_Position = vec4(Out.noteSize.x * pos.x + noteShift,pos.y, 0.0, 1.0);
    //         }
    //     "#;
    // let fragment_shader_src = r#"
    //     #version 330

    //     out vec3 fragColor;

    //     in INTERFACE {
    //         vec2 uv;
    //         vec2 noteSize;
    //     } In;

    //     #define cornerRadius 0.01

    //     void main() {
    //         vec3 baseColor = 1.35*vec3(0.770, 0.257, 1.323);

    //         // Rounded corner (super-ellipse equation).
    //         float radiusPosition = pow(abs(In.uv.x/(0.5*In.noteSize.x)), In.noteSize.x/cornerRadius) + pow(abs(In.uv.y/(0.5*In.noteSize.y)), In.noteSize.y/cornerRadius);

    //         if(	radiusPosition > 1.0){
    //             discard;
    //         }

    //         // Fragment color.
    //         fragColor = (0.8+0.2*(1.0-1.0))*baseColor;

    //         if(	radiusPosition > 0.8){
    //             fragColor *= 1.05;
    //         }
    //     }
    // "#;

    // let program = glium::Program::new(
    //     &display,
    //     glium::program::ProgramCreationInput::SourceCode {
    //         vertex_shader: vertex_shader_src,
    //         fragment_shader: fragment_shader_src,
    //         geometry_shader: None,
    //         tessellation_control_shader: None,
    //         tessellation_evaluation_shader: None,
    //         transform_feedback_varyings: None,
    //         outputs_srgb: true,
    //         uses_point_size: false,
    //     },
    // )
    // .unwrap();

    let start_time = std::time::SystemTime::now();

    let mut closed = false;
    while !closed {
        // let time = std::time::SystemTime::now()
        //     .duration_since(start_time)
        //     .unwrap()
        //     .as_millis() as f64
        //     / 1000.0;

        // notes_on.retain(|no| {
        //     let delete = {
        //         if time >= no.start + no.duration {
        //             conn_out.send(&[0x80, no.note, no.vel]).unwrap();
        //             println!("{}{}", " ".repeat(no.note as usize), "#".red().bold());
        //             true
        //         } else {
        //             false
        //         }
        //     };
        //     !delete
        // });


        // if time >= note.start {
        //     // println!("{} \t T:{} \t {}", note.note, note.track_id, time);
        //     println!(
        //         "{}{}{:.2}",
        //         " ".repeat(note.note as usize),
        //         "#".green().bold(),
        //         note.start
        //     );
        //     if note.ch != 9 {
        //         conn_out.send(&[0x90, note.note, note.vel]).unwrap();
        //         notes_on.push(note);
        //     }

        //     index += 1;
        //     if index >= midi.merged_track.notes.len() {
        //         // TODO: Break After all notes are off in notes_on vec;
        //         // Temporary solution, stop all notes befor break
        //         for no in notes_on {
        //             conn_out.send(&[0x80, no.note, no.vel]).unwrap();
        //         }
        //         break;
        //     }
        //     note = &midi.merged_track.notes[index];
        // }


        game_renderer.draw();
        // let mut target = display.draw();
        // target.clear_color_srgb(10.0 / 255.0, 42.0 / 255.0, 89.0 / 255.0, 1.0);
        // target
        //     .draw(
        //         &vertex_buffer,
        //         &indices,
        //         &program,
        //         &uniform! {t:note.note as f32 - 21.0},
        //         &Default::default(),
        //     )
        //     .unwrap();

        // target.finish().unwrap();
        events_loop.poll_events(|ev| match ev {
            glutin::Event::WindowEvent { event, .. } => match event {
                glutin::WindowEvent::Resized(_size) => {
                    // We can't use glutin size becouse of winit problem
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


                    // let pox_x = f64::max(
                    //     f64::min(1.0, 2.0 * mpx / game_renderer.window_w as f64 - 1.0),
                    //     -1.0,
                    // );
                    // let pox_y = -f64::max(
                    //     f64::min(1.0, 2.0 * position.y / game_renderer.window_h as f64 - 1.0),
                    //     -1.0,
                    // );


                    game_renderer.m_pos_x = pox_x;
                    game_renderer.m_pos_y = pox_y;

                    // println!("{:.2},{:.2}", pox_x, pox_y);
                    // game_renderer
                }
                glutin::WindowEvent::CloseRequested => closed = true,
                _ => (),
            },
            _ => (),
        });
    }

    // let start_time = std::time::SystemTime::now();

    // let mut index = 0;
    // if midi.merged_track.notes.len() == 0 {
    //     panic!(
    //         "No Notes In Track For Some Reason \n {:?}",
    //         midi.merged_track
    //     )
    // }
    // let mut note = &midi.merged_track.notes[index];

    // let mut notes_on: Vec<&lib_midi::track::MidiNote> = Vec::new();
    // loop {
    //     let time = std::time::SystemTime::now()
    //         .duration_since(start_time)
    //         .unwrap()
    //         .as_millis() as f64
    //         / 1000.0;


    //     notes_on.retain(|no| {
    //         let delete = {
    //             if time >= no.start + no.duration {
    //                 conn_out.send(&[0x80, no.note, no.vel]).unwrap();
    //                 println!("{}{}", " ".repeat(no.note as usize), "#".red().bold());
    //                 true
    //             } else {
    //                 false
    //             }
    //         };
    //         !delete
    //     });


    //     if time >= note.start {
    //         // println!("{} \t T:{} \t {}", note.note, note.track_id, time);
    //         println!(
    //             "{}{}{:.2}",
    //             " ".repeat(note.note as usize),
    //             "#".green().bold(),
    //             note.start
    //         );
    //         if note.ch != 9 {
    //             conn_out.send(&[0x90, note.note, note.vel]).unwrap();
    //             notes_on.push(note);
    //         }

    //         index += 1;
    //         if index >= midi.merged_track.notes.len() {
    //             // TODO: Break After all notes are off in notes_on vec;
    //             // Temporary solution, stop all notes befor break
    //             for no in notes_on {
    //                 conn_out.send(&[0x80, no.note, no.vel]).unwrap();
    //             }
    //             break;
    //         }
    //         note = &midi.merged_track.notes[index];
    //     }
    // }

}
