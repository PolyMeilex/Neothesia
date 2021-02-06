#![allow(dead_code)]

mod wgpu_jumpstart;
use wgpu_jumpstart::{Gpu, Uniform, Window};

mod ui;

mod scene;

mod time_manager;

mod output_manager;
pub use output_manager::OutputManager;

mod transform_uniform;
use transform_uniform::TransformUniform;

mod config;

mod rectangle_pipeline;

mod resources;

mod target;

mod main_state;

#[cfg(not(feature = "record"))]
mod app;

#[cfg(feature = "record")]
mod recorder;

use futures::Future;
use winit::event_loop::EventLoop;

#[cfg(not(feature = "record"))]
fn run_app() {
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

    let mut app = app::App::new(gpu, window);

    // Investigate:
    // https://github.com/gfx-rs/wgpu-rs/pull/306

    event_loop.run(move |event, _, control_flow| {
        app.target.window.on_event(&mut app.target.gpu, &event);

        use winit::event::Event;
        match &event {
            Event::MainEventsCleared => {
                let event = app.game_scene.main_events_cleared(&mut app.target);
                app.scene_event(event, control_flow);

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

fn main() {
    #[cfg(not(feature = "record"))]
    run_app();
    #[cfg(feature = "record")]
    run_recorder();
}

#[cfg(feature = "record")]
fn run_recorder() {
    use env_logger::Env;
    env_logger::Builder::from_env(Env::default().default_filter_or("neothesia=info")).init();
    let event_loop = EventLoop::new();

    let winit_window = winit::window::WindowBuilder::new()
        .with_title("Neothesia")
        .with_inner_size(winit::dpi::LogicalSize {
            width: 1920,
            height: 1080,
        })
        .with_visible(false);

    #[cfg(target_os = "windows")]
    let winit_window = {
        use winit::platform::windows::WindowBuilderExtWindows;
        winit_window.with_drag_and_drop(false)
    };

    let winit_window = winit_window.build(&event_loop).unwrap();

    let (window, gpu) = block_on(Window::new(winit_window)).unwrap();

    let mut recorder = recorder::Recorder::new(gpu, window);

    {
        recorder.resize();
        let texture_desc = wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width: 1920,
                height: 1080,
                depth: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu_jumpstart::TEXTURE_FORMAT,
            usage: wgpu::TextureUsage::COPY_SRC | wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            label: None,
        };
        let texture = recorder.target.gpu.device.create_texture(&texture_desc);
        let view = &texture.create_view(&wgpu::TextureViewDescriptor {
            label: None,
            format: None,
            dimension: None,
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            level_count: None,
            base_array_layer: 0,
            array_layer_count: None,
        });

        let u32_size = std::mem::size_of::<u32>() as u32;
        let output_buffer_size = (u32_size * 1920 * 1080) as wgpu::BufferAddress;

        let output_buffer_desc = wgpu::BufferDescriptor {
            size: output_buffer_size,
            usage: wgpu::BufferUsage::COPY_DST | wgpu::BufferUsage::MAP_READ,
            label: None,
            mapped_at_creation: false,
        };

        std::fs::create_dir("./out").ok();
        let mut encoder = mpeg_encoder::Encoder::new("./out/video.mp4", 1920, 1080);

        encoder.init(Some(0.0),Some("medium"));


        let start = std::time::Instant::now();

        let mut n = 1;
        while recorder.scene.playback_progress() < 101.0 {
            let output_buffer = recorder
                .target
                .gpu
                .device
                .create_buffer(&output_buffer_desc);

            recorder.update();
            recorder.render(&texture, &view, &texture_desc, &output_buffer);

            {
                let slice = output_buffer.slice(..);
                block_on(async {
                    let task = slice.map_async(wgpu::MapMode::Read);

                    recorder.target.gpu.device.poll(wgpu::Maintain::Wait);

                    task.await.unwrap();

                    let mapping = slice.get_mapped_range();

                    let data: &[u8] = &mapping;
                    encoder.encode_bgra(1920, 1080, data, false);
                    println!(
                        "Encoded {} frames ({}s, {}%) in {}s",
                        n,
                        (n as f32 / 60.0).round(),
                        recorder.scene.playback_progress().round(),
                        start.elapsed().as_secs()
                    );
                });
            }

            n += 1;
        }
    }
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
