mod bg_pipeline;
mod iced_menu;

use bg_pipeline::BgPipeline;
use iced_menu::IcedMenu;

use winit::event::WindowEvent;

use crate::{
    scene::{Scene, SceneEvent, SceneType},
    time_manager::Timer,
    ui::iced_conversion,
    MainState, Target,
};

#[derive(Debug)]
pub enum Event {
    Play,
}

pub struct MenuScene {
    bg_pipeline: BgPipeline,
    timer: Timer,
    iced_state: iced_native::program::State<IcedMenu>,

    main_state: MainState,
}

impl MenuScene {
    pub fn new(target: &mut Target, mut state: MainState) -> Self {
        let timer = Timer::new();

        let menu = IcedMenu::new(
            std::mem::replace(&mut state.midi_file, None),
            state.output_manager.get_outputs(),
            state.output_manager.selected_output_id,
            state.output_manager.selected_font_path.clone(),
        );
        let iced_state = iced_native::program::State::new(
            menu,
            target.iced_manager.viewport.logical_size(),
            iced_conversion::cursor_position(
                target.window.state.cursor_physical_position,
                target.iced_manager.viewport.scale_factor(),
            ),
            &mut target.iced_manager.renderer,
            &mut target.iced_manager.debug,
        );

        let mut scene = Self {
            bg_pipeline: BgPipeline::new(&target.gpu),
            timer,
            iced_state,

            main_state: state,
        };

        scene.resize(target);
        scene
    }
}

impl Scene for MenuScene {
    fn done(self: Box<Self>) -> MainState {
        self.main_state
    }

    fn scene_type(&self) -> SceneType {
        SceneType::MainMenu
    }

    fn update(&mut self, target: &mut Target) -> SceneEvent {
        self.timer.update();
        let time = self.timer.get_elapsed() / 1000.0;

        self.bg_pipeline.update_time(&mut target.gpu, time);

        let outs = self.main_state.output_manager.get_outputs();
        self.iced_state
            .queue_message(iced_menu::Message::OutputsUpdated(outs));

        SceneEvent::None
    }

    fn render(&mut self, target: &mut Target, frame: &wgpu::SwapChainFrame) {
        let encoder = &mut target.gpu.encoder;
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &frame.output.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });
            self.bg_pipeline.render(&mut render_pass);
        }

        let _mouse_interaction = target.iced_manager.renderer.backend_mut().draw(
            &target.gpu.device,
            &mut target.gpu.staging_belt,
            &mut target.gpu.encoder,
            &frame.output.view,
            &target.iced_manager.viewport,
            self.iced_state.primitive(),
            &target.iced_manager.debug.overlay(),
        );
    }

    fn window_event(&mut self, target: &mut Target, event: &WindowEvent) -> SceneEvent {
        let modifiers = winit::event::ModifiersState::default();

        if let Some(event) = iced_conversion::window_event(
            &event,
            target.iced_manager.viewport.scale_factor(),
            modifiers,
        ) {
            self.iced_state.queue_event(event);
        }

        if let winit::event::WindowEvent::KeyboardInput { input, .. } = &event {
            if let winit::event::ElementState::Released = input.state {
                if let Some(key) = input.virtual_keycode {
                    match key {
                        winit::event::VirtualKeyCode::Space => self
                            .iced_state
                            .queue_message(iced_menu::Message::FileSelectPressed),
                        winit::event::VirtualKeyCode::Left => self
                            .iced_state
                            .queue_message(iced_menu::Message::PrevPressed),
                        winit::event::VirtualKeyCode::Right => self
                            .iced_state
                            .queue_message(iced_menu::Message::NextPressed),
                        winit::event::VirtualKeyCode::Return => self
                            .iced_state
                            .queue_message(iced_menu::Message::PlayPressed),
                        winit::event::VirtualKeyCode::Escape => return SceneEvent::GoBack,
                        _ => {}
                    }
                }
            }
        }

        SceneEvent::None
    }

    fn main_events_cleared(&mut self, target: &mut Target) -> SceneEvent {
        if !self.iced_state.is_queue_empty() {
            let event = self.iced_state.update(
                target.iced_manager.viewport.logical_size(),
                iced_conversion::cursor_position(
                    target.window.state.cursor_physical_position,
                    target.iced_manager.viewport.scale_factor(),
                ),
                None,
                &mut target.iced_manager.renderer,
                &mut target.iced_manager.debug,
            );

            if let Some(event) = event {
                for f in event.futures() {
                    let event = crate::block_on(async { f.await });

                    match event {
                        iced_menu::Message::MainMenuDone(midi, out) => {
                            let program = self.iced_state.program();

                            self.main_state.midi_file = Some(midi);

                            self.main_state.output_manager.selected_output_id =
                                Some(program.carousel.id());
                            self.main_state.output_manager.connect(out);

                            return SceneEvent::MainMenu(Event::Play);
                        }
                        _ => {}
                    }
                }
            }
        }

        SceneEvent::None
    }
}
