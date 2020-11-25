use crate::{
    midi_device::MidiPortInfo,
    scene::{Scene, SceneEvent, SceneType},
    time_manager::Timer,
    ui::iced_conversion,
    Target,
};

use super::{bg_pipeline::BgPipeline, iced_menu, IcedMenu};

use winit::event::WindowEvent;

#[derive(Debug)]
pub enum Event {
    MidiOpen(MidiPortInfo),
}

pub struct MenuScene {
    bg_pipeline: BgPipeline,
    timer: Timer,
    iced_state: iced_native::program::State<IcedMenu>,
}

impl MenuScene {
    pub fn new(target: &mut Target) -> Self {
        let timer = Timer::new();

        let menu = IcedMenu::new(target.state.midi_file.clone());
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

        match &event {
            winit::event::WindowEvent::KeyboardInput { input, .. } => match input.virtual_keycode {
                Some(winit::event::VirtualKeyCode::Return) => {
                    if let winit::event::ElementState::Released = input.state {
                        self.iced_state
                            .queue_message(iced_menu::Message::PlayPressed)
                    }
                }
                Some(winit::event::VirtualKeyCode::Escape) => {
                    if let winit::event::ElementState::Released = input.state {
                        return SceneEvent::GoBack;
                    }
                }
                _ => {}
            },
            _ => {}
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
                        iced_menu::Message::MainMenuDone(f, p) => {
                            target.state.midi_file = Some(f);

                            return SceneEvent::MainMenu(Event::MidiOpen(p.unwrap()));
                        }
                        _ => {}
                    }
                }
            }
        }

        SceneEvent::None
    }
}
