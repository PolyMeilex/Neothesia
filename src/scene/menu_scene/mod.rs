mod bg_pipeline;
mod iced_menu;

mod neo_btn;

use bg_pipeline::BgPipeline;
use iced_menu::IcedMenu;

use winit::event::WindowEvent;

use crate::{
    scene::{Scene, SceneEvent, SceneType},
    target::Target,
    time_manager::Timer,
    ui::iced_conversion,
};

#[derive(Debug)]
pub enum Event {
    Play,
}

pub struct MenuScene {
    bg_pipeline: BgPipeline,
    timer: Timer,
    iced_state: iced_native::program::State<IcedMenu>,
}

impl MenuScene {
    pub fn new(target: &mut Target) -> Self {
        let timer = Timer::new();

        let menu = IcedMenu::new(&mut target.state);
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
        };

        scene.resize(target);
        scene
    }
}

impl Scene for MenuScene {
    fn scene_type(&self) -> SceneType {
        SceneType::MainMenu
    }

    fn update(&mut self, target: &mut Target) -> SceneEvent {
        self.timer.update();
        let time = self.timer.get_elapsed() / 1000.0;

        self.bg_pipeline.update_time(&mut target.gpu, time);

        let outs = target.state.output_manager.get_outputs();
        self.iced_state
            .queue_message(iced_menu::Message::OutputsUpdated(outs));

        SceneEvent::None
    }

    fn render(&mut self, target: &mut Target, view: &wgpu::TextureView) {
        let encoder = &mut target.gpu.encoder;
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: view,
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
            view,
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
                        winit::event::VirtualKeyCode::Tab => self
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
                            .queue_message(iced_menu::Message::EnterPressed),
                        // winit::event::VirtualKeyCode::Escape => return SceneEvent::GoBack,
                        winit::event::VirtualKeyCode::Escape => self
                            .iced_state
                            .queue_message(iced_menu::Message::EscPressed),
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
                        iced_menu::Message::OutputFileSelected(path) => {
                            let midi = lib_midi::Midi::new(path.to_str().unwrap());

                            if let Err(e) = &midi {
                                log::error!("{}", e);
                            }

                            target.state.midi_file = midi.ok();

                            self.iced_state
                                .queue_message(iced_menu::Message::MidiFileUpdate(
                                    target.state.midi_file.is_some(),
                                ));
                        }
                        iced_menu::Message::OutputMainMenuDone(out) => {
                            let program = self.iced_state.program();

                            #[cfg(feature = "play_along")]
                            {
                                target.state.config.play_along = program.play_along;
                            }

                            target.state.output_manager.selected_output_id =
                                Some(program.carousel.id());
                            target.state.output_manager.connect(out);

                            return SceneEvent::MainMenu(Event::Play);
                        }
                        iced_menu::Message::OutputAppExit => {
                            return SceneEvent::GoBack;
                        }
                        _ => {}
                    }
                }
            }
        }

        SceneEvent::None
    }
}
