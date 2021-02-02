mod wgpu_jumpstart;
use futures::Future;

use wgpu_jumpstart::{Gpu, Uniform, Window};

mod ui;

mod scene;

mod time_manager;

mod output_manager;
pub use output_manager::OutputManager;

mod transform_uniform;
use transform_uniform::TransformUniform;

mod config;

use winit::{event::Event, event_loop::EventLoop};

mod rectangle_pipeline;

mod resources;

mod target;

mod app;
use app::{App, MainState};

fn main() {
    {
        #[cfg(not(target_arch = "wasm32"))]
        {
            use env_logger::Env;
            env_logger::Builder::from_env(Env::default().default_filter_or("neothesia=info"))
                .init();
        }

        #[cfg(target_arch = "wasm32")]
        {
            console_log::init().expect("could not initialize logger");
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        }
    }

    let event_loop = EventLoop::new();

    let winit_window = winit::window::WindowBuilder::new()
        .with_title("Neothesia")
        .with_inner_size(winit::dpi::LogicalSize {
            width: 1080.0,
            height: 720.0,
        });

    #[cfg(target_os = "windows")]
    let winit_window = {
        use winit::platform::windows::WindowBuilderExtWindows;
        winit_window.with_drag_and_drop(false)
    };

    let winit_window = winit_window.build(&event_loop).unwrap();

    let (window, gpu) = block_on(Window::new(winit_window)).unwrap();

    let mut app = App::new(gpu, window);

    // Commented out control_flow stuff is related to:
    // https://github.com/gfx-rs/wgpu-rs/pull/306
    // I think it messes with my framerate so for now it's commented out, needs more testing

    // #[cfg(not(target_arch = "wasm32"))]
    // let mut last_update_inst = std::time::Instant::now();
    event_loop.run(move |event, _, control_flow| {
        // *control_flow = {
        //     #[cfg(not(target_arch = "wasm32"))]
        //     {
        //         ControlFlow::WaitUntil(
        //             std::time::Instant::now() + std::time::Duration::from_millis(10),
        //         )
        //     }
        //     #[cfg(target_arch = "wasm32")]
        //     {
        //         ControlFlow::Poll
        //     }
        // };

        app.target.window.on_event(&mut app.target.gpu, &event);

        match &event {
            Event::MainEventsCleared => {
                // #[cfg(not(target_arch = "wasm32"))]
                // {
                //     if last_update_inst.elapsed() > std::time::Duration::from_millis(20) {
                //         app.window.request_redraw();
                //         last_update_inst = std::time::Instant::now();
                //     }
                // }

                let event = app.game_scene.main_events_cleared(&mut app.target);
                app.scene_event(event, control_flow);

                // #[cfg(target_arch = "wasm32")]
                app.target.window.request_redraw();
            }
            Event::WindowEvent { event, .. } => {
                app.window_event(event, control_flow);
            }
            Event::RedrawRequested(_) => {
                app.update(control_flow);
                app.render();
            }
            _ => {}
        }
    });
}

pub fn block_on<F>(f: F) -> <F as Future>::Output
where
    F: Future,
{
    #[cfg(not(target_arch = "wasm32"))]
    return futures::executor::block_on(f);
    #[cfg(target_arch = "wasm32")]
    return wasm_bindgen_futures::spawn_local(f);
}
