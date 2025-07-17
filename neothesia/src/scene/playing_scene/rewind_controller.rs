use std::time::Duration;

use winit::{
    dpi::PhysicalPosition,
    event::{ElementState, WindowEvent},
};

use super::MidiPlayer;
use crate::{context::Context, utils::window::WindowState};

pub enum RewindController {
    Keyboard { speed: i64, was_paused: bool },
    Mouse { was_paused: bool },
    None,
}

impl RewindController {
    pub fn new() -> Self {
        Self::None
    }

    pub fn is_rewinding(&self) -> bool {
        !matches!(self, RewindController::None)
    }

    pub fn start_mouse_rewind(&mut self, player: &mut MidiPlayer) {
        let was_paused = player.is_paused();
        self.start_rewind(player, RewindController::Mouse { was_paused });
    }

    fn start_keyboard_rewind(&mut self, player: &mut MidiPlayer, speed: i64) {
        let was_paused = player.is_paused();
        self.start_rewind(player, RewindController::Keyboard { speed, was_paused });
    }

    fn start_rewind(&mut self, player: &mut MidiPlayer, controller: RewindController) {
        player.pause();
        *self = controller;
    }

    pub fn stop_rewind(&mut self, player: &mut MidiPlayer) {
        let controller = std::mem::replace(self, RewindController::None);

        let was_paused = match controller {
            RewindController::Keyboard { was_paused, .. } => Some(was_paused),
            RewindController::Mouse { was_paused } => Some(was_paused),
            RewindController::None => None,
        };

        if was_paused == Some(false) {
            player.resume();
        }
    }

    #[profiling::function]
    pub fn update(&self, player: &mut MidiPlayer, ctx: &Context, delta: Duration) {
        if let RewindController::Keyboard { speed, .. } = self {
            let v = if ctx.window_state.modifiers_state.shift_key() {
                *speed * 2
            } else if ctx.window_state.modifiers_state.control_key() {
                *speed / 2
            } else {
                *speed
            };

            player.rewind((100.0 * v as f32 * delta.as_secs_f32()).round() as i64);
        }
    }

    pub fn handle_window_event(
        &mut self,
        ctx: &mut Context,
        event: &WindowEvent,
        player: &mut MidiPlayer,
    ) {
        match &event {
            WindowEvent::KeyboardInput { event, .. } => {
                self.handle_keyboard_input(player, event);
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.handle_cursor_moved(player, &ctx.window_state, position);
            }
            _ => {}
        }
    }

    fn handle_keyboard_input(&mut self, player: &mut MidiPlayer, input: &winit::event::KeyEvent) {
        use winit::keyboard::{Key, NamedKey};

        let Key::Named(name) = input.logical_key else {
            return;
        };

        match name {
            NamedKey::ArrowLeft => match input.state {
                ElementState::Pressed => {
                    if !self.is_rewinding() {
                        self.start_keyboard_rewind(player, -100);
                    }
                }
                ElementState::Released => {
                    if let RewindController::Keyboard { .. } = self {
                        self.stop_rewind(player);
                    }
                }
            },
            NamedKey::ArrowRight => match input.state {
                ElementState::Pressed => {
                    if !self.is_rewinding() {
                        self.start_keyboard_rewind(player, 100);
                    }
                }
                ElementState::Released => {
                    if let RewindController::Keyboard { .. } = self {
                        self.stop_rewind(player);
                    }
                }
            },
            _ => {}
        }
    }

    fn handle_cursor_moved(
        &mut self,
        player: &mut MidiPlayer,
        window_state: &WindowState,
        position: &PhysicalPosition<f64>,
    ) {
        if let RewindController::Mouse { .. } = self {
            let x = position.to_logical::<f32>(window_state.scale_factor).x;
            let w = &window_state.logical_size.width;

            let p = x / w;
            player.set_percentage_time(p);
        }
    }
}
