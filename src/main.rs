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
    let mut last_time_fps = std::time::Instant::now();

    //Delta Time
    let mut last_time = std::time::Instant::now();
    let mut delta_time = 0;

    let mut time_elapsed: u128 = 0;


    while !closed {
        time_elapsed += last_time.elapsed().as_millis();
        last_time = std::time::Instant::now();


        // FPS
        fps += 1.0;
        if last_time_fps.elapsed().as_millis() > 1000 {
            last_time_fps = std::time::Instant::now();

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
                glutin::WindowEvent::MouseWheel { delta, .. } => {
                    if let glutin::MouseScrollDelta::LineDelta(x, y) = delta {
                        if let game_states::GameStateType::playing_state =
                            &game_renderer.get_state_type()
                        {
                            let val = y as i32 * 100;

                            if val > 0 {
                                time_elapsed += val as u128;
                            } else if val < 0 {
                                let val = val.abs() as u128;

                                if time_elapsed >= val {
                                    time_elapsed -= val;
                                }
                            }

                            time_elapsed += y as u128 * 100;
                            last_time = std::time::Instant::now();
                        }
                    }
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
