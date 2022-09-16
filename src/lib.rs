#![allow(clippy::collapsible_match, clippy::single_match)]

use target::Target;
pub use wgpu_jumpstart::{Gpu, TransformUniform, Uniform, Window};

pub mod ui;

pub mod scene;

pub mod utils;

pub mod output_manager;
pub use output_manager::OutputManager;

pub mod input_manager;

pub mod config;

pub mod target;

pub mod midi_event;
use midi_event::MidiEvent;

use futures::Future;
use winit::event_loop::{EventLoop, EventLoopBuilder};

#[derive(Debug)]
pub enum NeothesiaEvent {
    #[cfg(feature = "app")]
    MainMenu(crate::scene::menu_scene::Event),
    MidiInput(MidiEvent),
    GoBack,
}

pub fn init(builder: winit::window::WindowBuilder) -> (EventLoop<NeothesiaEvent>, Target) {
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

    let event_loop = EventLoopBuilder::with_user_event().build();
    let proxy = event_loop.create_proxy();

    let builder = builder.with_title("Neothesia");

    #[cfg(target_os = "windows")]
    let builder = {
        use winit::platform::windows::WindowBuilderExtWindows;
        builder.with_drag_and_drop(false)
    };

    #[cfg(any(
        target_os = "linux",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd"
    ))]
    let builder = {
        use winit::platform::unix::WindowBuilderExtUnix;
        builder.with_wayland_csd_theme(winit::window::Theme::Dark)
    };

    let winit_window = builder.build(&event_loop).unwrap();

    let (window, gpu) = block_on(Window::new(winit_window)).unwrap();

    let target = Target::new(window, proxy, gpu);

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
