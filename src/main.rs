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
    pub midi_file: Option<Arc<lib_midi::Midi>>,
}

impl MainState {
    fn new(gpu: &Gpu) -> Self {
        let args: Vec<String> = std::env::args().collect();

        let midi_file = if args.len() > 1 {
            if let Some(midi) = lib_midi::Midi::new(&args[1]).ok() {
                Some(Arc::new(midi))
            } else {
                None
            }
        } else {
            None
        };

        Self { midi_file }
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

        let physical_size = window.state.physical_size;
        let viewport = iced_wgpu::Viewport::with_physical_size(
            iced_native::Size::new(physical_size.width, physical_size.height),
            window.state.scale_factor,
        );

        Self {
            renderer,
            viewport,
            debug,
        }
    }
}

pub struct Target {
    pub state: MainState,

    pub window: Window,
    pub gpu: Gpu,
    pub transform_uniform: Uniform<TransformUniform>,

    pub ui: Ui,
    pub iced_manager: IcedManager,
}

impl Target {
    pub fn new(window: Window, mut gpu: Gpu) -> Self {
        let state = MainState::new(&gpu);

        let transform_uniform = Uniform::new(
            &gpu.device,
            TransformUniform::default(),
            wgpu::ShaderStage::VERTEX,
        );

        let ui = Ui::new(&transform_uniform, &mut gpu);

        let iced_manager = IcedManager::new(&gpu.device, &window);

        Self {
            state,

            window,
            gpu,
            transform_uniform,

            ui,
            iced_manager,
        }
    }
    fn resize(&mut self) {
        {
            let winit::dpi::LogicalSize { width, height } = self.window.state.logical_size;
            self.transform_uniform.data.update(width, height);
            self.transform_uniform
                .update(&mut self.gpu.encoder, &self.gpu.device);
        }

        {
            let physical_size = self.window.state.physical_size;
            self.iced_manager.viewport = iced_wgpu::Viewport::with_physical_size(
                iced_native::Size::new(physical_size.width, physical_size.height),
                self.window.state.scale_factor,
            );
        }
    }
}

struct App {
    target: Target,

    fps_timer: Fps,
    game_scene: Box<scene::scene_transition::SceneTransition>,
}

impl App {
    fn new(gpu: Gpu, window: Window) -> Self {
        let mut target = Target::new(window, gpu);

        let game_scene = scene::menu_scene::MenuScene::new(&mut target);
        let mut game_scene = Box::new(scene::scene_transition::SceneTransition::new(Box::new(
            game_scene,
        )));

        target.resize();
        game_scene.resize(&mut target);
        target.gpu.submit().unwrap();

        Self {
            target,
            fps_timer: Fps::new(),
            game_scene,
        }
    }

    fn window_event(&mut self, event: &WindowEvent, control_flow: &mut ControlFlow) {
        match &event {
            WindowEvent::Resized(_) => {
                self.target.resize();
                self.game_scene.resize(&mut self.target);

                self.target.gpu.submit().unwrap();
            }
            WindowEvent::ScaleFactorChanged { .. } => {
                // TODO: Check if this update is needed;
                self.target.resize();
                self.game_scene.resize(&mut self.target);
            }
            WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
            _ => {}
        }

        let scene_event = self.game_scene.window_event(&mut self.target, event);
        self.scene_event(scene_event, control_flow);
    }

    fn scene_event(&mut self, event: SceneEvent, control_flow: &mut ControlFlow) {
        match event {
            SceneEvent::MainMenu(event) => match event {
                scene::menu_scene::Event::MidiOpen(port) => {
                    let state = scene::playing_scene::PlayingScene::new(&mut self.target, port);
                    self.game_scene.transition_to(Box::new(state));
                }
            },
            SceneEvent::GoBack => match self.game_scene.scene_type() {
                SceneType::MainMenu => {
                    *control_flow = ControlFlow::Exit;
                }
                SceneType::Playing => {
                    let state = scene::menu_scene::MenuScene::new(&mut self.target);
                    self.game_scene.transition_to(Box::new(state));
                }
                SceneType::Transition => {}
            },
            _ => {}
        }
    }

    fn update(&mut self, control_flow: &mut ControlFlow) {
        self.fps_timer.update();

        let event = self.game_scene.update(&mut self.target);

        self.scene_event(event, control_flow);

        self.queue_fps();
    }

    fn render(&mut self) {
        let frame = self
            .target
            .window
            .get_current_frame()
            .expect("Could not get_current_frame()");

        self.target.gpu.clear(&frame);

        self.game_scene.render(&mut self.target, &frame);

        // let _mouse_interaction = self.main_state.iced_manager.renderer.backend_mut().draw(
        //     &mut self.gpu.device,
        //     &mut self.gpu.encoder,
        //     &frame.view,
        //     &self.main_state.iced_manager.viewport,
        //     self.main_state.iced_manager.state.primitive(),
        //     &self.main_state.iced_manager.debug.overlay(),
        // );

        self.target.ui.render(
            &self.target.window,
            &self.target.transform_uniform,
            &self.target.state,
            &mut self.target.gpu,
            &frame,
        );

        self.target.gpu.submit().unwrap();
    }

    fn queue_fps(&mut self) {
        let s = format!("FPS: {}", self.fps_timer.fps());
        let text = vec![wgpu_glyph::Text::new(&s)
            .with_color([1.0, 1.0, 1.0, 1.0])
            .with_scale(20.0)];

        self.target.ui.queue_text(Section {
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
}

fn main_async() {
    let event_loop = EventLoop::new();

    let winit_window = winit::window::WindowBuilder::new()
        .with_title("Neothesia")
        .with_inner_size(winit::dpi::LogicalSize {
            width: 1080.0,
            height: 720.0,
        })
        .build(&event_loop)
        .unwrap();

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
