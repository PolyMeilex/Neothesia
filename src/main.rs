mod wgpu_jumpstart;
use wgpu_jumpstart::{Gpu, Uniform, Window};

mod ui;
use ui::Ui;

mod scene;
use scene::{InputEvent, Scene, SceneEvent, SceneType};

mod time_menager;
use time_menager::TimeMenager;

mod midi_device;

mod transform_uniform;
use transform_uniform::TransformUniform;

use wgpu_glyph::Section;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};

use std::rc::Rc;

mod rectangle_pipeline;

#[cfg(feature = "record")]
mod recorder;

pub struct MainState {
    pub window_size: (f32, f32),
    pub mouse_pos: (f32, f32),
    /// Mouse Was Clicked This Frame
    pub mouse_clicked: bool,
    /// Mouse Is Pressed This Frame
    pub mouse_pressed: bool,
    pub time_menager: TimeMenager,
    pub transform_uniform: Uniform<TransformUniform>,

    pub midi_file: Option<Rc<lib_midi::Midi>>,
}

impl MainState {
    fn new(gpu: &Gpu) -> Self {
        Self {
            window_size: (0.0, 0.0),
            mouse_pos: (0.0, 0.0),
            mouse_clicked: false,
            mouse_pressed: false,
            time_menager: TimeMenager::new(),
            transform_uniform: Uniform::new(
                &gpu.device,
                TransformUniform::default(),
                wgpu::ShaderStage::VERTEX,
            ),
            midi_file: None,
        }
    }
    fn resize(&mut self, gpu: &mut Gpu, w: f32, h: f32) {
        self.window_size = (w, h);
        self.transform_uniform.data.update(w, h);
        self.transform_uniform.update(&mut gpu.encoder, &gpu.device);
    }
    fn update_mouse_pos(&mut self, x: f32, y: f32) {
        self.mouse_pos = (x, y);
    }
    fn update_mouse_pressed(&mut self, state: bool) {
        self.mouse_pressed = state;

        if state {
            self.update_mouse_clicked(true);
        }
    }
    fn update_mouse_clicked(&mut self, clicked: bool) {
        self.mouse_clicked = clicked;
    }
}

enum AppEvent<'a> {
    WindowEvent(&'a WindowEvent<'a>, &'a mut ControlFlow),
    SceneEvent(SceneEvent),
}

struct App {
    pub window: Window,
    pub gpu: Gpu,
    pub ui: Ui,
    pub main_state: MainState,
    game_scene: Box<scene::scene_transition::SceneTransition>,

    #[cfg(feature = "record")]
    recorder: recorder::Recorder,
}

impl App {
    fn new(mut gpu: Gpu, window: Window) -> Self {
        let mut main_state = MainState::new(&gpu);

        let ui = Ui::new(&main_state, &mut gpu);

        #[cfg(not(feature = "record"))]
        let game_scene = scene::menu_scene::MenuScene::new(&mut main_state, &mut gpu);
        #[cfg(feature = "record")]
        let game_scene = {
            let midi = Rc::new(
                lib_midi::Midi::new(
                    "/home/poly/Documents/Midi/Zero no Tsukaima Season 2 OP - I Say Yes.mid",
                )
                .expect("midi error!"),
            );

            main_state.midi_file = Some(midi);

            let game_scene =
                scene::playing_scene::PlayingScene::new(&mut gpu, &mut main_state, None);

            game_scene
        };

        let game_scene = Box::new(scene::scene_transition::SceneTransition::new(Box::new(
            game_scene,
        )));

        Self {
            window,
            gpu,
            ui,
            main_state,
            game_scene,

            #[cfg(feature = "record")]
            recorder: recorder::Recorder::new(),
        }
    }
    fn event(&mut self, event: AppEvent) {
        match event {
            AppEvent::WindowEvent(event, control_flow) => {
                match event {
                    WindowEvent::Resized(_) => {
                        self.resize();
                        self.gpu.submit();
                    }
                    WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                        self.window.on_dpi(*scale_factor);
                        // TODO: Check if this update is needed;
                        self.resize();
                    }
                    WindowEvent::CursorMoved { position, .. } => {
                        let dpi = &self.window.dpi;
                        let x = (position.x / dpi).round();
                        let y = (position.y / dpi).round();

                        self.main_state.update_mouse_pos(x as f32, y as f32);

                        let ae = AppEvent::SceneEvent(self.game_scene.input_event(
                            &mut self.main_state,
                            InputEvent::CursorMoved(x as f32, y as f32),
                        ));
                        self.event(ae);
                    }
                    WindowEvent::MouseInput { state, button, .. } => {
                        if let winit::event::ElementState::Pressed = state {
                            self.main_state.update_mouse_pressed(true);
                        } else {
                            self.main_state.update_mouse_pressed(false);
                        }

                        let ae = AppEvent::SceneEvent(self.game_scene.input_event(
                            &mut self.main_state,
                            InputEvent::MouseInput(state, button),
                        ));
                        self.event(ae);
                    }
                    WindowEvent::Focused(_) => {
                        self.main_state.update_mouse_pressed(false);
                    }
                    WindowEvent::KeyboardInput { input, .. } => {
                        if input.state == winit::event::ElementState::Released {
                            match input.virtual_keycode {
                                Some(winit::event::VirtualKeyCode::Escape) => {
                                    self.go_back(control_flow);
                                }
                                Some(key) => {
                                    let ae = AppEvent::SceneEvent(self.game_scene.input_event(
                                        &mut self.main_state,
                                        InputEvent::KeyReleased(key),
                                    ));
                                    self.event(ae);
                                }
                                _ => {}
                            }
                        }
                    }
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    _ => {}
                }
            }
            AppEvent::SceneEvent(event) => match event {
                SceneEvent::MainMenu(event) => match event {
                    scene::menu_scene::Event::MidiOpen(port) => {
                        let state = scene::playing_scene::PlayingScene::new(
                            &mut self.gpu,
                            &mut self.main_state,
                            port,
                        );
                        self.game_scene.transition_to(Box::new(state));

                        // Self::render() is called right after this so we need to update scene here to prevent transition from not being visible for the first frame
                        self.game_scene
                            .update(&mut self.main_state, &mut self.gpu, &mut self.ui);
                    }
                },
                _ => {}
            },
        }
    }
    fn resize(&mut self) {
        self.window.on_resize(&mut self.gpu);
        let (w, h) = self.window.size();

        self.main_state.resize(&mut self.gpu, w, h);
        self.game_scene.resize(&mut self.main_state, &mut self.gpu);
        self.ui.resize(&self.main_state, &mut self.gpu);
    }
    fn go_back(&mut self, control_flow: &mut ControlFlow) {
        match self.game_scene.scene_type() {
            SceneType::MainMenu => {
                *control_flow = ControlFlow::Exit;
            }
            SceneType::Playing => {
                let state = scene::menu_scene::MenuScene::new(&mut self.main_state, &mut self.gpu);
                self.game_scene.transition_to(Box::new(state));
            }
            SceneType::Transition => {}
        }
    }
    fn update(&mut self) {
        self.main_state.time_menager.update();

        let event = self
            .game_scene
            .update(&mut self.main_state, &mut self.gpu, &mut self.ui);

        self.event(AppEvent::SceneEvent(event));

        self.queue_fps();
    }
    #[cfg(not(feature = "record"))]
    fn render(&mut self) {
        let frame = self.window.surface.get_next_texture();
        let view = &frame.view;

        {
            self.clear(view);

            self.game_scene
                .render(&mut self.main_state, &mut self.gpu, view);

            self.ui.render(&mut self.main_state, &mut self.gpu, view);

            self.gpu.submit();

            self.main_state.update_mouse_clicked(false);
        }
    }
    #[cfg(feature = "record")]
    fn render<'a>(
        &mut self,
        texture: &wgpu::Texture,
        view: &wgpu::TextureView,
        texture_desc: &wgpu::TextureDescriptor<'a>,
        output_buffer: &wgpu::Buffer,
        n: usize,
    ) /*-> Vec<u8>*/
    {
        {
            self.clear(view);

            self.game_scene
                .render(&mut self.main_state, &mut self.gpu, view);

            self.ui.render(&mut self.main_state, &mut self.gpu, view);

            self.main_state.update_mouse_clicked(false);
        }

        {
            let u32_size = std::mem::size_of::<u32>() as u32;

            self.gpu.encoder.copy_texture_to_buffer(
                wgpu::TextureCopyView {
                    texture: &texture,
                    mip_level: 0,
                    array_layer: 0,
                    origin: wgpu::Origin3d::ZERO,
                },
                wgpu::BufferCopyView {
                    buffer: &output_buffer,
                    offset: 0,
                    bytes_per_row: u32_size * 1920,
                    rows_per_image: 1080,
                },
                texture_desc.size,
            );

            self.gpu.submit();

            // let mapping = output_buffer.map_read(0, output_buffer_size);
            // self.gpu.device.poll(wgpu::Maintain::Wait);

            // let result = mapping.await.unwrap();
            // let data: &[u8] = result.as_slice();

            // let out = data.to_owned();

            // out.resize(1280 * 720 * 4, 0);

            // out.clone_from_slice(data);

            // self.recorder.encoder.encode_rgba(1280, 720, data, false);
            // out

            // self.recorder.encoder.encode_rgba(1280, 720, data, false);
            // use image::{Bgra, ImageBuffer};
            // let buffer = ImageBuffer::<Bgra<u8>, &[u8]>::from_raw(1280, 720, data).unwrap();

            // buffer
            //     .save(&format!(
            //         "/home/poly/Documents/Programing/rust/Neothesia/out/image{}.jpg",
            //         n
            //     ))
            //     .unwrap();
        }
    }
    fn queue_fps(&mut self) {
        let s = format!("FPS: {}", self.main_state.time_menager.fps());
        let text = vec![wgpu_glyph::Text::new(&s)
            .with_color([1.0, 1.0, 1.0, 1.0])
            .with_scale(20.0)];

        self.ui.queue_text(Section {
            text,
            screen_position: (0.0, 5.0),
            layout: wgpu_glyph::Layout::Wrap {
                line_breaker: Default::default(),
                h_align: wgpu_glyph::HorizontalAlign::Left,
                v_align: wgpu_glyph::VerticalAlign::Top,
            },
            ..Default::default()
        });
    }
    fn clear(&mut self, view: &wgpu::TextureView) {
        self.gpu
            .encoder
            .begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: view,
                    resolve_target: None,
                    load_op: wgpu::LoadOp::Clear,
                    store_op: wgpu::StoreOp::Store,
                    clear_color: wgpu::Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 1.0,
                    },
                }],
                depth_stencil_attachment: None,
            });
    }
}

async fn main_async() {
    let event_loop = EventLoop::new();

    let builder = winit::window::WindowBuilder::new().with_title("Neothesia");
    let (window, gpu) = Window::new(builder, (1280, 720), &event_loop).await;
    let mut app = App::new(gpu, window);
    app.resize();
    app.gpu.submit();

    #[cfg(feature = "record")]
    {
        app.resize();

        let texture_desc = wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width: 1920,
                height: 1080,
                depth: 1,
            },
            array_layer_count: 1,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu_jumpstart::TEXTURE_FORMAT,
            usage: wgpu::TextureUsage::COPY_SRC | wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            label: None,
        };
        let texture = app.gpu.device.create_texture(&texture_desc);
        let view = &texture.create_default_view();

        let u32_size = std::mem::size_of::<u32>() as u32;
        let output_buffer_size = (u32_size * 1920 * 1080) as wgpu::BufferAddress;

        let output_buffer_desc = wgpu::BufferDescriptor {
            size: output_buffer_size,
            usage: wgpu::BufferUsage::COPY_DST | wgpu::BufferUsage::MAP_READ,
            label: None,
        };

        let (tx, rx) = std::sync::mpsc::channel::<Option<wgpu::Buffer>>();

        let d = app.gpu.device.clone();
        let th = std::thread::spawn(move || {
            async fn run(
                rx: std::sync::mpsc::Receiver<Option<wgpu::Buffer>>,
                device: &wgpu::Device,
            ) {
                let u32_size = std::mem::size_of::<u32>() as u32;
                let output_buffer_size = (u32_size * 1920 * 1080) as wgpu::BufferAddress;
                let mut encoder = mpeg_encoder::Encoder::new(
                    "/home/poly/Documents/Programing/rust/Neothesia/out/test.mp4",
                    1280,
                    720,
                );
                encoder.init();

                loop {
                    if let Ok(b) = rx.recv() {
                        if let Some(output_buffer) = b {
                            let mapping = output_buffer.map_read(0, output_buffer_size);
                            device.poll(wgpu::Maintain::Wait);

                            let result = mapping.await.unwrap();
                            let data: &[u8] = result.as_slice();
                            encoder.encode_bgra(1920, 1080, data, false);
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }
            }
            futures::executor::block_on(run(rx, &d));
        });

        let t = std::time::Instant::now();

        for n in 0..(60 * 10) {
            app.update();

            let output_buffer = app.gpu.device.create_buffer(&output_buffer_desc);

            app.render(&texture, &view, &texture_desc, &output_buffer, n);

            tx.send(Some(output_buffer)).ok();
        }

        log::info!("End: {}", t.elapsed().as_millis());

        tx.send(None).ok();
        th.join().unwrap();

        log::info!("End: {}", t.elapsed().as_millis());
    }

    // Commented out control_flow stuff is related to:
    // https://github.com/gfx-rs/wgpu-rs/pull/306
    // I think it messes with my framerate so for now it's commented out, needs more testing

    // #[cfg(not(target_arch = "wasm32"))]
    // let mut last_update_inst = std::time::Instant::now();

    #[cfg(not(feature = "record"))]
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
        match &event {
            Event::MainEventsCleared => {
                // #[cfg(not(target_arch = "wasm32"))]
                // {
                //     if last_update_inst.elapsed() > std::time::Duration::from_millis(20) {
                //         app.window.request_redraw();
                //         last_update_inst = std::time::Instant::now();
                //     }
                // }

                // #[cfg(target_arch = "wasm32")]
                app.window.request_redraw();
            }
            Event::WindowEvent { event, .. } => {
                app.event(AppEvent::WindowEvent(event, control_flow));
            }
            Event::RedrawRequested(_) => {
                app.update();
                app.render();
            }
            _ => {}
        }
    });
}

fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    {
        use env_logger::Env;
        // env_logger::init();
        env_logger::from_env(Env::default().default_filter_or("neothesia=info")).init();
        futures::executor::block_on(main_async());
    }

    #[cfg(target_arch = "wasm32")]
    {
        console_log::init().expect("could not initialize logger");
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));

        wasm_bindgen_futures::spawn_local(main_async());
    }
}
