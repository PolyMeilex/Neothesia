use crate::{
    midi_device::{MidiDevicesManager, MidiPortInfo},
    scene::{InputEvent, Scene, SceneEvent, SceneType},
    time_manager::Timer,
    ui::Ui,
    wgpu_jumpstart::{Color, Gpu},
    MainState,
};

use super::{bg_pipeline::BgPipeline, iced_menu, IcedMenu};

use std::{rc::Rc, sync::Arc};
use winit::event::{MouseButton, VirtualKeyCode, WindowEvent};

#[derive(Debug, Clone)]
pub enum Event {
    MidiOpen(MidiPortInfo),
}

pub struct MenuScene {
    bg_pipeline: BgPipeline,
    timer: Timer,
    iced_state: iced_native::program::State<IcedMenu>,
}

impl MenuScene {
    pub fn new(state: &mut MainState, gpu: &mut Gpu) -> Self {
        let timer = Timer::new();

        let menu = IcedMenu::new(state.midi_file.clone());
        let iced_state = iced_native::program::State::new(
            menu,
            state.iced_manager.viewport.logical_size(),
            crate::iced_conversion::cursor_position(
                state.cursor_physical_position,
                state.iced_manager.viewport.scale_factor(),
            ),
            &mut state.iced_manager.renderer,
            &mut state.iced_manager.debug,
        );

        let mut scene = Self {
            bg_pipeline: BgPipeline::new(&gpu),
            timer,
            iced_state,
        };

        scene.resize(state, gpu);
        scene
    }
}

impl Scene for MenuScene {
    fn scene_type(&self) -> SceneType {
        SceneType::MainMenu
    }
    fn update(&mut self, _state: &mut MainState, gpu: &mut Gpu, _ui: &mut Ui) -> SceneEvent {
        self.timer.update();
        let time = self.timer.get_elapsed() / 1000.0;

        self.bg_pipeline.update_time(gpu, time);

        SceneEvent::None
    }
    fn render(&mut self, main_state: &mut MainState, gpu: &mut Gpu, frame: &wgpu::SwapChainOutput) {
        let encoder = &mut gpu.encoder;
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &frame.view,
                    resolve_target: None,
                    load_op: wgpu::LoadOp::Load,
                    store_op: wgpu::StoreOp::Store,
                    clear_color: wgpu::Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 0.0,
                    },
                }],
                depth_stencil_attachment: None,
            });
            self.bg_pipeline.render(&mut render_pass);
        }

        let _mouse_interaction = main_state.iced_manager.renderer.backend_mut().draw(
            &mut gpu.device,
            &mut gpu.encoder,
            &frame.view,
            &main_state.iced_manager.viewport,
            self.iced_state.primitive(),
            &main_state.iced_manager.debug.overlay(),
        );
    }
    fn input_event(&mut self, _state: &mut MainState, event: InputEvent) -> SceneEvent {
        match event {
            InputEvent::KeyReleased(key) => match key {
                VirtualKeyCode::Return => SceneEvent::None,
                _ => SceneEvent::None,
            },
            InputEvent::MouseInput(s, button) => match button {
                MouseButton::Left => {
                    if let winit::event::ElementState::Pressed = s {
                        // self.mouse_clicked(state)
                        SceneEvent::None
                    } else {
                        SceneEvent::None
                    }
                }
                _ => SceneEvent::None,
            },
            InputEvent::CursorMoved(x, y) => {
                // self.update_mouse_pos(x, y);
                SceneEvent::None
            } // _ => SceneEvent::None,
        }
    }
    fn window_event(&mut self, main_state: &mut MainState, event: &WindowEvent) -> SceneEvent {
        let modifiers = winit::event::ModifiersState::default();

        if let Some(event) = crate::iced_conversion::window_event(
            &event,
            main_state.iced_manager.viewport.scale_factor(),
            modifiers,
        ) {
            self.iced_state.queue_event(event);
        }

        SceneEvent::None
    }
    fn main_events_cleared(&mut self, main_state: &mut MainState) -> SceneEvent {
        if !self.iced_state.is_queue_empty() {
            let event = self.iced_state.update(
                main_state.iced_manager.viewport.logical_size(),
                crate::iced_conversion::cursor_position(
                    main_state.cursor_physical_position,
                    main_state.iced_manager.viewport.scale_factor(),
                ),
                None,
                &mut main_state.iced_manager.renderer,
                &mut main_state.iced_manager.debug,
            );

            if let Some(event) = event {
                for f in event.futures() {
                    let event = crate::block_on(async { f.await });

                    match event {
                        iced_menu::Message::MainMenuDone(f, p) => {
                            main_state.midi_file = Some(f);

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
