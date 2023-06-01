use std::time::Duration;

use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

use wgpu_jumpstart::Gpu;

async fn run(event_loop: EventLoop<()>, window: Window) {
    let midi = midi_file::Midi::new_from_bytes(include_bytes!("../../test.mid")).unwrap();
    let mut playback = midi_file::PlaybackState::new(Duration::from_secs(3), &midi.merged_track);

    let size = window.inner_size();
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());

    let surface = unsafe { instance.create_surface(&window) }.unwrap();
    let mut gpu = Gpu::new(&instance, Some(&surface)).await.unwrap();

    let width = size.width;
    let height = size.height;

    let mut transform_uniform = wgpu_jumpstart::TransformUniform::default();
    transform_uniform.update(width as f32, height as f32, 1.0);

    let transform_uniform = wgpu_jumpstart::Uniform::new(
        &gpu.device,
        transform_uniform,
        wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
    );

    let keyboard_layout = get_layout(width as f32, height as f32);

    let mut keyboard = neothesia_core::render::KeyboardRenderer::new(
        &gpu,
        &transform_uniform,
        keyboard_layout.clone(),
    );

    keyboard.position_on_bottom_of_parent(height as f32);

    let neothesia_config = neothesia_core::config::Config::new();
    let mut waterfall = neothesia_core::render::WaterfallRenderer::new(
        &gpu,
        &midi,
        &neothesia_config,
        &transform_uniform,
        keyboard_layout,
    );

    let mut text = neothesia_core::render::TextRenderer::new(&gpu);

    // keyboard.update(&gpu.queue, text.glyph_brush());

    //

    // Load the shaders from disk

    let capabilities = surface.get_capabilities(&gpu.adapter);
    let swapchain_format = capabilities.formats[0];

    let mut config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: swapchain_format,
        view_formats: vec![swapchain_format],
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::Fifo,
        alpha_mode: capabilities.alpha_modes[0],
    };

    surface.configure(&gpu.device, &config);

    let mut last_time = instant::Instant::now();

    event_loop.run(move |event, _, control_flow| {
        // Have the closure take ownership of the resources.
        // `event_loop.run` never returns, therefore we must do this to ensure
        // the resources are properly cleaned up.
        let _ = (&instance, &gpu.adapter);

        match event {
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                // Reconfigure the surface with the new size
                config.width = size.width;
                config.height = size.height;
                surface.configure(&gpu.device, &config);
                // On macos the window needs to be redrawn manually after resizing
                window.request_redraw();
            }
            Event::RedrawRequested(_) => {
                let frame = surface
                    .get_current_texture()
                    .expect("Failed to acquire next swap chain texture");
                let view = frame
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());
                let mut encoder = gpu
                    .device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
                {
                    let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: None,
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Load,
                                store: true,
                            },
                        })],
                        depth_stencil_attachment: None,
                    });

                    waterfall.render(&transform_uniform, &mut rpass);
                    keyboard.render(&transform_uniform, &mut rpass);
                }

                text.render((width as f32, height as f32), &mut gpu, &view);

                gpu.queue.submit(Some(encoder.finish()));
                frame.present();
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            Event::MainEventsCleared => {
                let delta = last_time.elapsed();
                last_time = instant::Instant::now();

                let events = playback.update(&midi.merged_track, delta);
                file_midi_events(&mut keyboard, &neothesia_config, &events);

                waterfall.update(&gpu.queue, time_without_lead_in(&playback));

                keyboard.update(&gpu.queue, text.glyph_brush());

                window.request_redraw();
            }
            _ => {}
        }
    });
}

fn main() {
    let event_loop = EventLoop::new();
    let window = winit::window::Window::new(&event_loop).unwrap();
    #[cfg(not(target_arch = "wasm32"))]
    {
        env_logger::init();
        // Temporarily avoid srgb formats for the swapchain on the web
        pollster::block_on(run(event_loop, window));
    }
    #[cfg(target_arch = "wasm32")]
    {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        console_log::init().expect("could not initialize logger");
        use winit::platform::web::WindowExtWebSys;

        on_resize(&window);

        // On wasm, append the canvas to the document body
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| doc.body())
            .and_then(|body| {
                body.append_child(&web_sys::Element::from(window.canvas()))
                    .ok()
            })
            .expect("couldn't append canvas to document body");
        wasm_bindgen_futures::spawn_local(run(event_loop, window));
    }
}

#[cfg(target_arch = "wasm32")]
fn on_resize(window: &Window) {
    use winit::dpi::{LogicalSize, PhysicalSize};

    let body = web_sys::window()
        .and_then(|win| win.document())
        .and_then(|doc| doc.body());

    if let Some(body) = body {
        let (width, height) = (body.client_width(), body.client_height());

        let factor = window.scale_factor();
        let logical = LogicalSize { width, height };
        let size: PhysicalSize<f32> = logical.to_physical(factor);

        window.set_inner_size(size)
    }
}

fn get_layout(width: f32, height: f32) -> piano_math::KeyboardLayout {
    let white_count = piano_math::KeyboardRange::standard_88_keys().white_count();
    let neutral_width = width / white_count as f32;
    let neutral_height = height * 0.2;

    piano_math::standard_88_keys(neutral_width, neutral_height)
}

pub fn file_midi_events(
    keyboard: &mut neothesia_core::render::KeyboardRenderer,
    config: &neothesia_core::config::Config,
    events: &[midi_file::MidiEvent],
) {
    use midi_file::midly::MidiMessage;

    for e in events {
        let (is_on, key) = match e.message {
            MidiMessage::NoteOn { key, .. } => (true, key.as_int()),
            MidiMessage::NoteOff { key, .. } => (false, key.as_int()),
            _ => continue,
        };

        if keyboard.range().contains(key) && e.channel != 9 {
            let id = key as usize - 21;
            let key = &mut keyboard.key_states_mut()[id];

            if is_on {
                let color = &config.color_schema[e.track_id % config.color_schema.len()];
                key.pressed_by_file_on(color);
            } else {
                key.pressed_by_file_off();
            }

            keyboard.queue_reupload();
        }
    }
}

pub fn time_without_lead_in(playback: &midi_file::PlaybackState) -> f32 {
    playback.time().as_secs_f32() - playback.leed_in().as_secs_f32()
}
