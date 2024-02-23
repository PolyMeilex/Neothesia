use winit::dpi::LogicalPosition;
use winit::dpi::PhysicalPosition;
use winit::keyboard::ModifiersState;

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
        }
    }

    pub fn window_event(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::Resized(ps) => {
                self.physical_size = *ps;
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
            _ => {}
        }
    }
}
