use winit::{
    dpi::PhysicalPosition,
    event::{ElementState, KeyboardInput, MouseButton, VirtualKeyCode},
};

use super::MidiPlayer;
use crate::{output_manager::OutputManager, target::Target};

pub enum RewindController {
    Keyboard { speed: i64, was_paused: bool },
    Mouse { was_paused: bool },
    None,
}

pub fn update(player: &mut MidiPlayer, target: &mut Target) {
    if let RewindController::Keyboard { speed, .. } = player.rewind_controller {
        if target.window.state.modifers_state.shift() {
            player.rewind(target, speed * 2);
        } else if target.window.state.modifers_state.ctrl() {
            player.rewind(target, speed / 2);
        } else {
            player.rewind(target, speed);
        }
    }
}

pub fn handle_keyboard_input(
    player: &mut MidiPlayer,
    output: &mut OutputManager,
    input: &KeyboardInput,
) {
    if let Some(virtual_keycode) = input.virtual_keycode {
        match virtual_keycode {
            VirtualKeyCode::Left => {
                if let winit::event::ElementState::Pressed = input.state {
                    if !player.is_rewinding() {
                        player.start_keyboard_rewind(output, -100);
                    }
                } else if let RewindController::Keyboard { .. } = player.rewind_controller() {
                    player.stop_rewind();
                }
            }
            VirtualKeyCode::Right => {
                if let winit::event::ElementState::Pressed = input.state {
                    if !player.is_rewinding() {
                        player.start_keyboard_rewind(output, 100);
                    }
                } else if let RewindController::Keyboard { .. } = player.rewind_controller() {
                    player.stop_rewind();
                }
            }
            _ => {}
        }
    }
}

pub fn handle_mouse_input(
    player: &mut MidiPlayer,
    target: &mut Target,
    state: &ElementState,
    button: &MouseButton,
) {
    if let (ElementState::Pressed, MouseButton::Left) = (state, button) {
        let pos = &target.window.state.cursor_logical_position;

        if pos.y < 20.0 && !player.is_rewinding() {
            player.start_mouse_rewind(&mut target.output_manager);

            let x = target.window.state.cursor_logical_position.x;
            let w = target.window.state.logical_size.width;

            let p = x / w;
            log::debug!("Progressbar: x:{},p:{}", x, p);
            player.set_percentage_time(target, p);
        }
    } else if let (ElementState::Released, MouseButton::Left) = (state, button) {
        if let RewindController::Mouse { .. } = player.rewind_controller() {
            player.stop_rewind();
        }
    }
}

pub fn handle_cursor_moved(
    player: &mut MidiPlayer,
    target: &mut Target,
    position: &PhysicalPosition<f64>,
) {
    if let RewindController::Mouse { .. } = player.rewind_controller() {
        let x = position
            .to_logical::<f32>(target.window.state.scale_factor)
            .x;
        let w = &target.window.state.logical_size.width;

        let p = x / w;
        log::debug!("Progressbar: x:{},p:{}", x, p);
        player.set_percentage_time(target, p);
    }
}

impl MidiPlayer {
    fn is_rewinding(&self) -> bool {
        !matches!(self.rewind_controller, RewindController::None)
    }

    fn start_mouse_rewind(&mut self, output: &mut OutputManager) {
        let was_paused = self.is_paused();
        self.start_rewind(output, RewindController::Mouse { was_paused });
    }

    fn start_keyboard_rewind(&mut self, output: &mut OutputManager, speed: i64) {
        let was_paused = self.is_paused();
        self.start_rewind(output, RewindController::Keyboard { speed, was_paused });
    }

    fn start_rewind(&mut self, output: &mut OutputManager, controller: RewindController) {
        self.pause(output);
        self.rewind_controller = controller;
    }

    fn stop_rewind(&mut self) {
        let controller = std::mem::replace(&mut self.rewind_controller, RewindController::None);

        let was_paused = match controller {
            RewindController::Keyboard { was_paused, .. } => Some(was_paused),
            RewindController::Mouse { was_paused } => Some(was_paused),
            RewindController::None => None,
        };

        if was_paused == Some(false) {
            self.resume();
        }
    }

    fn rewind_controller(&self) -> &RewindController {
        &self.rewind_controller
    }
}
