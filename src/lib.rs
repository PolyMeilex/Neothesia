pub mod wgpu_jumpstart;
use target::Target;
pub use wgpu_jumpstart::{Gpu, TransformUniform, Uniform, Window};

pub mod ui;

pub mod scene;

pub mod utils;

pub mod output_manager;
pub use output_manager::OutputManager;

pub mod config;

pub mod quad_pipeline;

pub mod target;

pub mod main_state;

use futures::Future;
use winit::event_loop::EventLoop;

pub fn init(builder: winit::window::WindowBuilder) -> (EventLoop<()>, Target) {
    #[cfg(not(target_arch = "wasm32"))]
    {
        use env_logger::Env;
        env_logger::Builder::from_env(Env::default().default_filter_or("neothesia=info")).init();
    }

    #[cfg(target_arch = "wasm32")]
    {
        console_log::init().expect("could not initialize logger");
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    }

    let event_loop = EventLoop::new();

    let builder = builder.with_title("Neothesia");

    #[cfg(target_os = "windows")]
    let builder = {
        use winit::platform::windows::WindowBuilderExtWindows;
        builder.with_drag_and_drop(false)
    };

    let winit_window = builder.build(&event_loop).unwrap();

    let (window, gpu) = block_on(Window::new(winit_window)).unwrap();

    let target = Target::new(window, gpu);

    (event_loop, target)
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
