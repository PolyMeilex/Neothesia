use winit::{
    dpi::{LogicalPosition, PhysicalPosition},
    event::{ElementState, KeyEvent, MouseButton},
    keyboard::{Key, ModifiersState},
};

use winit::{
    dpi::{LogicalSize, PhysicalSize},
    event::WindowEvent,
};

pub struct WindowState {
    pub physical_size: PhysicalSize<u32>,
    pub logical_size: LogicalSize<f32>,

    pub scale_factor: f64,

    pub cursor_physical_position: PhysicalPosition<f64>,
    pub cursor_logical_position: LogicalPosition<f32>,

    pub focused: bool,

    pub modifiers_state: ModifiersState,
    pub left_mouse_btn: bool,
    pub right_mouse_btn: bool,
}

impl WindowState {
    pub fn new(window: &winit::window::Window) -> Self {
        let scale_factor = window.scale_factor();

        let (physical_size, logical_size) = {
            let physical_size = window.inner_size();
            let logical_size = physical_size.to_logical::<f32>(scale_factor);
            (physical_size, logical_size)
        };

        let cursor_physical_position = PhysicalPosition::new(0.0, 0.0);
        let cursor_logical_position = LogicalPosition::new(0.0, 0.0);

        Self {
            physical_size,
            logical_size,

            scale_factor,

            cursor_physical_position,
            cursor_logical_position,

            focused: false,

            modifiers_state: ModifiersState::default(),
            left_mouse_btn: false,
            right_mouse_btn: false,
        }
    }

    pub fn window_event(&mut self, event: &WindowEvent) {
        match event {
            // Windows sets size to 0 on minimise
            WindowEvent::Resized(ps) if ps.width > 0 && ps.height > 0 => {
                self.physical_size.width = ps.width;
                self.physical_size.height = ps.height;
                self.logical_size = ps.to_logical(self.scale_factor);
            }
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                self.logical_size = self.physical_size.to_logical(self.scale_factor);
                self.scale_factor = *scale_factor;
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor_physical_position = *position;
                self.cursor_logical_position = position.to_logical(self.scale_factor);
            }
            WindowEvent::Focused(f) => {
                self.focused = *f;
            }
            WindowEvent::ModifiersChanged(state) => {
                self.modifiers_state = state.state();
            }
            WindowEvent::MouseInput {
                state,
                button: MouseButton::Left,
                ..
            } => {
                self.left_mouse_btn = *state == ElementState::Pressed;
            }
            WindowEvent::MouseInput {
                state,
                button: MouseButton::Right,
                ..
            } => {
                self.right_mouse_btn = *state == ElementState::Pressed;
            }
            _ => {}
        }
    }
}

#[allow(unused)]
pub trait WinitEvent {
    fn scale_factor_changed(&self) -> bool;
    fn window_resized(&self) -> bool;
    fn cursor_moved(&self) -> bool;
    fn redraw_requested(&self) -> bool;

    fn mouse_pressed(&self, btn: MouseButton) -> bool;
    fn mouse_released(&self, btn: MouseButton) -> bool;

    fn left_mouse_pressed(&self) -> bool {
        self.mouse_pressed(MouseButton::Left)
    }

    fn left_mouse_released(&self) -> bool {
        self.mouse_released(MouseButton::Left)
    }

    fn right_mouse_pressed(&self) -> bool {
        self.mouse_pressed(MouseButton::Right)
    }

    fn right_mouse_released(&self) -> bool {
        self.mouse_released(MouseButton::Right)
    }

    fn back_mouse_pressed(&self) -> bool {
        self.mouse_pressed(MouseButton::Back)
    }

    fn back_mouse_released(&self) -> bool {
        self.mouse_released(MouseButton::Back)
    }

    fn key_pressed(&self, key: Key<&str>) -> bool;
    fn key_released(&self, key: Key<&str>) -> bool;

    fn character_released(&self) -> Option<&str>;
}

impl WinitEvent for WindowEvent {
    fn scale_factor_changed(&self) -> bool {
        matches!(self, Self::ScaleFactorChanged { .. })
    }

    fn window_resized(&self) -> bool {
        matches!(self, Self::Resized { .. })
    }

    fn cursor_moved(&self) -> bool {
        matches!(self, Self::CursorMoved { .. })
    }

    fn redraw_requested(&self) -> bool {
        matches!(self, Self::RedrawRequested { .. })
    }

    fn mouse_pressed(&self, btn: MouseButton) -> bool {
        match self {
            Self::MouseInput {
                state: ElementState::Pressed,
                button,
                ..
            } => button == &btn,
            _ => false,
        }
    }

    fn mouse_released(&self, btn: MouseButton) -> bool {
        match self {
            Self::MouseInput {
                state: ElementState::Released,
                button,
                ..
            } => button == &btn,
            _ => false,
        }
    }

    fn key_pressed(&self, key: Key<&str>) -> bool {
        match self {
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        state: ElementState::Pressed,
                        logical_key,
                        repeat: false,
                        ..
                    },
                ..
            } => logical_key.as_ref() == key,
            _ => false,
        }
    }

    fn key_released(&self, key: Key<&str>) -> bool {
        match self {
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        state: ElementState::Released,
                        logical_key,
                        repeat: false,
                        ..
                    },
                ..
            } => logical_key.as_ref() == key,
            _ => false,
        }
    }

    fn character_released(&self) -> Option<&str> {
        match self {
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        state: ElementState::Released,
                        logical_key: Key::Character(ch),
                        repeat: false,
                        ..
                    },
                ..
            } => Some(ch.as_str()),
            _ => None,
        }
    }
}
