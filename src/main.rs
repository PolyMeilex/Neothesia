mod wgpu_jumpstart;
use futures::Future;

use wgpu_jumpstart::{Gpu, Uniform, Window};

mod ui;
use ui::{IcedManager, TextRenderer};

mod scene;
use scene::{Scene, SceneEvent, SceneType};

mod time_manager;
use time_manager::Fps;

mod output_manager;
pub use output_manager::OutputManager;

mod transform_uniform;
use transform_uniform::TransformUniform;

use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};

mod rectangle_pipeline;

pub struct MainState {
    pub midi_file: Option<lib_midi::Midi>,
    pub output_manager: OutputManager,
}

impl MainState {
    fn new() -> Self {
        let args: Vec<String> = std::env::args().collect();

        let midi_file = if args.len() > 1 {
            if let Ok(midi) = lib_midi::Midi::new(&args[1]) {
                Some(midi)
            } else {
                None
            }
        } else {
            None
        };

        Self {
            midi_file,
            output_manager: OutputManager::new(),
        }
    }
}

pub struct Target {
    // pub state: MainState,
    pub window: Window,
    pub gpu: Gpu,
    pub transform_uniform: Uniform<TransformUniform>,

    pub text_renderer: TextRenderer,
    pub iced_manager: IcedManager,
}

impl Target {
    pub fn new(window: Window, gpu: Gpu) -> Self {
        let transform_uniform = Uniform::new(
            &gpu.device,
            TransformUniform::default(),
            wgpu::ShaderStage::VERTEX,
        );

        let text_renderer = TextRenderer::new(&gpu);

        let iced_manager = IcedManager::new(&gpu.device, &window);

        Self {
            // state,
            window,
            gpu,
            transform_uniform,

            text_renderer,
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
    game_scene: scene::scene_transition::SceneTransition,
}

impl App {
    fn new(gpu: Gpu, window: Window) -> Self {
        let mut target = Target::new(window, gpu);

        let state = MainState::new();

        let game_scene = scene::menu_scene::MenuScene::new(&mut target, state);
        let mut game_scene =
            scene::scene_transition::SceneTransition::new(Box::new(game_scene), &target);

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
                scene::menu_scene::Event::Play => {
                    let to = |target: &mut Target, state: MainState| -> Box<dyn Scene> {
                        let state = scene::playing_scene::PlayingScene::new(target, state);
                        Box::new(state)
                    };

                    let to = Box::new(to);

                    self.game_scene.transition_to(to);
                }
            },
            SceneEvent::GoBack => match self.game_scene.scene_type() {
                SceneType::MainMenu => {
                    *control_flow = ControlFlow::Exit;
                }
                SceneType::Playing => {
                    // let file = self.target.state.midi_file.clone();

                    let to = |target: &mut Target, state: MainState| -> Box<dyn Scene> {
                        let state = scene::menu_scene::MenuScene::new(target, state);
                        Box::new(state)
                    };

                    let to = Box::new(to);

                    self.game_scene.transition_to(to);
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

        self.target.text_renderer.queue_fps(self.fps_timer.fps());
    }

    fn render(&mut self) {
        let frame = self
            .target
            .window
            .get_current_frame()
            .expect("Could not get_current_frame()");

        self.target.gpu.clear(&frame);

        self.game_scene.render(&mut self.target, &frame);

        self.target
            .text_renderer
            .render(&self.target.window, &mut self.target.gpu, &frame);

        self.target.gpu.submit().unwrap();
    }
}

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

pub fn block_on<F>(f: F) -> <F as Future>::Output
where
    F: Future,
{
    #[cfg(not(target_arch = "wasm32"))]
    return futures::executor::block_on(f);
    #[cfg(target_arch = "wasm32")]
    return wasm_bindgen_futures::spawn_local(f);
}
