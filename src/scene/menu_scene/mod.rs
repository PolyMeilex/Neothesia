mod bg_pipeline;
mod iced_menu;

mod neo_btn;

use std::time::Duration;

use bg_pipeline::BgPipeline;
use iced_menu::IcedMenu;

use winit::event::WindowEvent;

use crate::{
    scene::{Scene, SceneType},
    target::Target,
    ui::{iced_conversion, iced_state, DummyClipboard},
    NeothesiaEvent,
};

#[derive(Debug)]
pub enum Event {
    Play,
}

pub struct MenuScene {
    bg_pipeline: BgPipeline,
    iced_state: iced_state::State<IcedMenu>,
}

impl MenuScene {
    pub fn new(target: &mut Target) -> Self {
        let menu = IcedMenu::new(target);
        let iced_state = iced_state::State::new(
            menu,
            target.iced_manager.viewport.logical_size(),
            &mut target.iced_manager.renderer,
        );

        let mut scene = Self {
            bg_pipeline: BgPipeline::new(&target.gpu),
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

    fn update(&mut self, target: &mut Target, delta: Duration) {
        self.bg_pipeline.update_time(&mut target.gpu, delta);

        let outs = target.output_manager.get_outputs();
        self.iced_state
            .queue_message(iced_menu::Message::OutputsUpdated(outs));
    }

    fn render(&mut self, target: &mut Target, view: &wgpu::TextureView) {
        let encoder = &mut target.gpu.encoder;
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            self.bg_pipeline.render(&mut render_pass);
        }

        let iced_renderer = &mut target.iced_manager.renderer;
        let device = &mut target.gpu.device;
        let staging_belt = &mut target.gpu.staging_belt;
        let encoder = &mut target.gpu.encoder;
        let viewport = &target.iced_manager.viewport;
        let overlay = &target.iced_manager.debug.overlay();

        iced_renderer.with_primitives(|backend, primitive| {
            backend.present(
                device,
                staging_belt,
                encoder,
                view,
                primitive,
                viewport,
                overlay,
            )
        })
    }

    fn window_event(&mut self, target: &mut Target, event: &WindowEvent) {
        use winit::event::{ElementState, ModifiersState, VirtualKeyCode};

        let modifiers = ModifiersState::default();

        if let Some(event) = iced_conversion::window_event(
            event,
            target.iced_manager.viewport.scale_factor(),
            modifiers,
        ) {
            self.iced_state.queue_event(event);
        }

        if let WindowEvent::KeyboardInput { input, .. } = &event {
            if let ElementState::Released = input.state {
                if let Some(key) = input.virtual_keycode {
                    match key {
                        VirtualKeyCode::Tab => self
                            .iced_state
                            .queue_message(iced_menu::Message::FileSelectPressed),
                        VirtualKeyCode::Left => self
                            .iced_state
                            .queue_message(iced_menu::Message::PrevPressed),
                        VirtualKeyCode::Right => self
                            .iced_state
                            .queue_message(iced_menu::Message::NextPressed),
                        VirtualKeyCode::Return => self
                            .iced_state
                            .queue_message(iced_menu::Message::EnterPressed),
                        VirtualKeyCode::Escape => self
                            .iced_state
                            .queue_message(iced_menu::Message::EscPressed),
                        _ => {}
                    }
                }
            }
        }
    }

    fn main_events_cleared(&mut self, target: &mut Target) {
        if !self.iced_state.is_queue_empty() {
            self.iced_state.update(target);
        }
    }
}
