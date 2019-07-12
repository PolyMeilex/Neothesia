extern crate lib_midi;
extern crate midir;

extern crate file_dialog;

mod game_states;
mod midi_device;
mod render;
mod utils;

#[macro_use]
extern crate glium;

fn main() {
    //let args: Vec<String> = std::env::args().collect();

    println!("Example Command: neothesia ~/my_midi_file.mid 1 (Id of midi output)");

    // let midi_out = MidiOutput::new("midi").unwrap();

    // println!("\nAvailable output ports:");
    // for i in 0..midi_out.port_count() {
    //     println!("{}: {}", i, midi_out.port_name(i).unwrap());
    // }

    // let out_port: usize;

    // if args.len() > 1 {
    //     out_port = args[1].parse::<usize>().unwrap();
    // } else {
    //     out_port = 0;
    // }

    // println!("Using Port Number {}", out_port);

    // let mut conn_out = midi_out.connect(out_port, "out").unwrap();

    //
    // Render
    //

    use glium::glutin;

    let mut events_loop = glutin::EventsLoop::new();
    let wb = glutin::WindowBuilder::new()
        .with_title("Neothesia!")
        .with_dimensions(glutin::dpi::LogicalSize::new(1280.0, 720.0));

    let cb = glutin::ContextBuilder::new().with_vsync(true);
    let display = glium::Display::new(wb, cb, &events_loop).unwrap();

    let mut game_renderer = render::GameRenderer::new(&display);

    let start_time = std::time::Instant::now();
    let mut closed = false;

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

            game_renderer.fps = fps as u64;

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
                    let pos_x = position.x;
                    let pos_y = position.y;

                    let cord =
                        utils::pixel_to_opengl(pos_x, pos_y, game_renderer.public_state.viewport);
                    game_renderer.public_state.m_pos = utils::Vec2 {
                        x: cord.x,
                        y: cord.y,
                    };
                }
                glutin::WindowEvent::MouseInput { state, .. } => match state {
                    glutin::ElementState::Pressed => {
                        game_renderer.public_state.m_pressed = true;
                        game_renderer.public_state.m_was_pressed = true;
                    }
                    glutin::ElementState::Released => game_renderer.public_state.m_pressed = false,
                },
                glutin::WindowEvent::KeyboardInput { input, .. } => match input.virtual_keycode {
                    Some(key) => match key {
                        glutin::VirtualKeyCode::Escape => closed = true,
                        _ => {}
                    },
                    None => {}
                },
                glutin::WindowEvent::CloseRequested => closed = true,
                _ => (),
            },
            _ => (),
        });
    }
}
