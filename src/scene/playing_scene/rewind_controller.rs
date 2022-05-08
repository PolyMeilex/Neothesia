use winit::{
    dpi::PhysicalPosition,
    event::{ElementState, KeyboardInput, MouseButton, VirtualKeyCode},
};

use crate::OutputManager;

use super::*;

pub enum RewindController {
    Keyboard { speed: i64, was_paused: bool },
    Mouse { was_paused: bool },
    None,
}

impl RewindController {
    pub fn update(&mut self, target: &mut Target, player: &mut MidiPlayer) {
        if let RewindController::Keyboard { speed, .. } = self {
            if target.window.state.modifers_state.shift() {
                player.rewind(target, *speed * 2);
            } else if target.window.state.modifers_state.ctrl() {
                player.rewind(target, *speed / 2);
            } else {
                player.rewind(target, *speed);
            }
        }
    }

    fn is_rewinding(&self) -> bool {
        !matches!(self, RewindController::None)
    }

    fn start_rewind(&mut self, controller: RewindController) {
        *self = controller;
    }

    fn stop_rewind(&mut self) -> Option<bool> {
        let controller = std::mem::replace(self, RewindController::None);

        match controller {
            RewindController::Keyboard { was_paused, .. } => Some(was_paused),
            RewindController::Mouse { was_paused } => Some(was_paused),
            RewindController::None => None,
        }
    }
}

impl PlayingScene {
    fn start_mouse_rewind(&mut self, output: &mut OutputManager) {
        let was_paused = self.player.is_paused();
        self.start_rewind(output, RewindController::Mouse { was_paused });
    }

    fn start_keyboard_rewind(&mut self, output: &mut OutputManager, speed: i64) {
        let was_paused = self.player.is_paused();
        self.start_rewind(output, RewindController::Keyboard { speed, was_paused });
    }

    fn start_rewind(&mut self, output: &mut OutputManager, controller: RewindController) {
        self.player.pause(output);
        self.rewind_controller.start_rewind(controller);
    }

    fn stop_rewind(&mut self) {
        let was_paused = self.rewind_controller.stop_rewind();
        if was_paused == Some(false) {
            self.player.resume();
        }
    }
}

impl PlayingScene {
    pub fn rewind_keyboard_input(&mut self, output: &mut OutputManager, input: &KeyboardInput) {
        if let Some(virtual_keycode) = input.virtual_keycode {
            match virtual_keycode {
                VirtualKeyCode::Left => {
                    if let winit::event::ElementState::Pressed = input.state {
                        if !self.rewind_controller.is_rewinding() {
                            self.start_keyboard_rewind(output, -100);
                        }
                    } else if let RewindController::Keyboard { .. } = &self.rewind_controller {
                        self.stop_rewind();
                    }
                }
                VirtualKeyCode::Right => {
                    if let winit::event::ElementState::Pressed = input.state {
                        if !self.rewind_controller.is_rewinding() {
                            self.start_keyboard_rewind(output, 100);
                        }
                    } else if let RewindController::Keyboard { .. } = &self.rewind_controller {
                        self.stop_rewind();
                    }
                }
                _ => {}
            }
        }
    }

    pub fn rewind_mouse_input(
        &mut self,
        target: &mut Target,
        state: &ElementState,
        button: &MouseButton,
    ) {
        if let (ElementState::Pressed, MouseButton::Left) = (state, button) {
            let pos = &target.window.state.cursor_logical_position;

            if pos.y < 20.0 && !self.rewind_controller.is_rewinding() {
                self.start_mouse_rewind(&mut target.output_manager.borrow_mut());

                let x = target.window.state.cursor_logical_position.x;
                let w = target.window.state.logical_size.width;

                let p = x / w;
                log::debug!("Progressbar: x:{},p:{}", x, p);
                self.player.set_percentage_time(target, p);
            }
        } else if let (ElementState::Released, MouseButton::Left) = (state, button) {
            if let RewindController::Mouse { .. } = &self.rewind_controller {
                self.stop_rewind();
            }
        }
    }

    pub fn rewind_handle_cursor_moved(
        &mut self,
        target: &mut Target,
        position: &PhysicalPosition<f64>,
    ) {
        if let RewindController::Mouse { .. } = &self.rewind_controller {
            let x = position
                .to_logical::<f32>(target.window.state.scale_factor)
                .x;
            let w = &target.window.state.logical_size.width;

            let p = x / w;
            log::debug!("Progressbar: x:{},p:{}", x, p);
            self.player.set_percentage_time(target, p);
        }
    }
}
