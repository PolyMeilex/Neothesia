mod wgpu_jumpstart;
use wgpu_jumpstart::{Gpu, Uniform, Window};

mod ui;
use ui::Ui;

mod scene;
use scene::{Scene, SceneEvent, SceneType};

mod time_manager;
use time_manager::Fps;

mod midi_device;

mod transform_uniform;
use transform_uniform::TransformUniform;

use wgpu_glyph::Section;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};

mod rectangle_pipeline;

mod iced_conversion;

pub struct MainState {
    pub cursor_physical_position: winit::dpi::PhysicalPosition<f64>,
    pub window_size: (f32, f32),
    pub mouse_pos: (f32, f32),
    /// Mouse Was Clicked This Frame
    pub mouse_clicked: bool,
    /// Mouse Is Pressed This Frame
    pub mouse_pressed: bool,
    pub transform_uniform: Uniform<TransformUniform>,

    pub midi_file: Option<Arc<lib_midi::Midi>>,

    pub iced_manager: IcedManager,
}

impl MainState {
    fn new(gpu: &Gpu, window: &Window) -> Self {
        let iced_manager = IcedManager::new(&gpu.device, &window);
        Self {
            cursor_physical_position: winit::dpi::PhysicalPosition::new(-1.0, -1.0),
            window_size: (0.0, 0.0),
            mouse_pos: (0.0, 0.0),
            mouse_clicked: false,
            mouse_pressed: false,
            transform_uniform: Uniform::new(
                &gpu.device,
                TransformUniform::default(),
                wgpu::ShaderStage::VERTEX,
            ),
            midi_file: None,
            iced_manager,
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

pub struct IcedManager {
    renderer: iced_wgpu::Renderer,
    viewport: iced_wgpu::Viewport,
    debug: iced_native::Debug,
}
impl IcedManager {
    fn new(device: &wgpu::Device, window: &Window) -> Self {
        let debug = iced_native::Debug::new();

        let mut settings = iced_wgpu::Settings::default();
        settings.format = wgpu_jumpstart::TEXTURE_FORMAT;

        let renderer = iced_wgpu::Renderer::new(iced_wgpu::Backend::new(device, settings));

        let physical_size = window.physical_size();
        let viewport = iced_wgpu::Viewport::with_physical_size(
            iced_native::Size::new(physical_size.width, physical_size.height),
            window.dpi,
        );

        Self {
            renderer,
            viewport,
            debug,
        }
    }
}

enum AppEvent<'a> {
    WindowEvent(&'a WindowEvent<'a>),
    SceneEvent(SceneEvent),
}

struct App {
    pub window: Window,
    pub gpu: Gpu,
    pub ui: Ui,
    pub main_state: MainState,
    fps_timer: Fps,
    game_scene: Box<scene::scene_transition::SceneTransition>,
}

impl App {
    fn new(mut gpu: Gpu, window: Window) -> Self {
        let mut main_state = MainState::new(&gpu, &window);

        let ui = Ui::new(&main_state, &mut gpu);
        let game_scene = scene::menu_scene::MenuScene::new(&mut main_state, &mut gpu);
        let game_scene = Box::new(scene::scene_transition::SceneTransition::new(Box::new(
            game_scene,
        )));

        Self {
            window,
            gpu,
            ui,
            main_state,
            fps_timer: Fps::new(),
            game_scene,
        }
    }

    fn event(&mut self, event: AppEvent, control_flow: &mut ControlFlow) {
        match event {
            AppEvent::WindowEvent(event) => {
                match &event {
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
                        self.main_state.cursor_physical_position = *position;

                        let dpi = &self.window.dpi;
                        let x = (position.x / dpi).round();
                        let y = (position.y / dpi).round();

                        self.main_state.update_mouse_pos(x as f32, y as f32);
                    }
                    WindowEvent::MouseInput { state, .. } => {
                        if let winit::event::ElementState::Pressed = state {
                            self.main_state.update_mouse_pressed(true);
                        } else {
                            self.main_state.update_mouse_pressed(false);
                        }
                    }
                    WindowEvent::Focused(_) => {
                        self.main_state.update_mouse_pressed(false);
                    }
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    _ => {}
                }

                let scene_event = self.game_scene.window_event(&mut self.main_state, event);
                self.event(AppEvent::SceneEvent(scene_event), control_flow)
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
                SceneEvent::GoBack => match self.game_scene.scene_type() {
                    SceneType::MainMenu => {
                        *control_flow = ControlFlow::Exit;
                    }
                    SceneType::Playing => {
                        let state =
                            scene::menu_scene::MenuScene::new(&mut self.main_state, &mut self.gpu);
                        self.game_scene.transition_to(Box::new(state));
                    }
                    SceneType::Transition => {}
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

        let physical_size = self.window.physical_size();
        self.main_state.iced_manager.viewport = iced_wgpu::Viewport::with_physical_size(
            iced_native::Size::new(physical_size.width, physical_size.height),
            self.window.dpi,
        );
    }

    fn update(&mut self, control_flow: &mut ControlFlow) {
        self.fps_timer.update();

        let event = self
            .game_scene
            .update(&mut self.main_state, &mut self.gpu, &mut self.ui);

        self.event(AppEvent::SceneEvent(event), control_flow);

        self.queue_fps();
    }

    fn render(&mut self) {
        let frame = self.window.surface.get_current_frame();

        self.clear(&frame);

        self.game_scene
            .render(&mut self.main_state, &mut self.gpu, &frame);

        // let _mouse_interaction = self.main_state.iced_manager.renderer.backend_mut().draw(
        //     &mut self.gpu.device,
        //     &mut self.gpu.encoder,
        //     &frame.view,
        //     &self.main_state.iced_manager.viewport,
        //     self.main_state.iced_manager.state.primitive(),
        //     &self.main_state.iced_manager.debug.overlay(),
        // );

        self.ui.render(&mut self.main_state, &mut self.gpu, &frame);

        self.gpu.submit();

        self.main_state.update_mouse_clicked(false);
    }

    fn queue_fps(&mut self) {
        let s = format!("FPS: {}", self.fps_timer.fps());
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

    fn clear(&mut self, frame: &wgpu::SwapChainFrame) {
        self.gpu
            .encoder
            .begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &frame.output.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });
    }
}

fn main_async() {
    let event_loop = EventLoop::new();

    let builder = winit::window::WindowBuilder::new().with_title("Neothesia");
    let (window, gpu) = block_on(Window::new(builder, (1080, 720), &event_loop));

    let mut app = App::new(gpu, window);
    app.resize();
    app.gpu.submit();

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
        match &event {
            Event::MainEventsCleared => {
                // #[cfg(not(target_arch = "wasm32"))]
                // {
                //     if last_update_inst.elapsed() > std::time::Duration::from_millis(20) {
                //         app.window.request_redraw();
                //         last_update_inst = std::time::Instant::now();
                //     }
                // }

                let event = app.game_scene.main_events_cleared(&mut app.main_state);
                app.event(AppEvent::SceneEvent(event), control_flow);

                // #[cfg(target_arch = "wasm32")]
                app.window.request_redraw();
            }
            Event::WindowEvent { event, .. } => {
                app.event(AppEvent::WindowEvent(event), control_flow);
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
    #[cfg(not(target_arch = "wasm32"))]
    {
        use env_logger::Env;
        // env_logger::init();
        env_logger::from_env(Env::default().default_filter_or("neothesia=info")).init();
        // futures::executor::block_on(main_async());
        main_async();
    }

    #[cfg(target_arch = "wasm32")]
    {
        console_log::init().expect("could not initialize logger");
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));

        // wasm_bindgen_futures::spawn_local(main_async());
        main_async()
    }
}

use std::{future::Future, sync::Arc};

pub fn block_on<F>(f: F) -> <F as Future>::Output
where
    F: Future,
{
    #[cfg(not(target_arch = "wasm32"))]
    return futures::executor::block_on(f);
    #[cfg(target_arch = "wasm32")]
    return wasm_bindgen_futures::spawn_local(f);
}
