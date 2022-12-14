use winit::dpi::LogicalPosition;
use winit::dpi::PhysicalPosition;
use winit::event::ModifiersState;

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

    pub modifers_state: ModifiersState,
}

impl WindowState {
    // #[cfg(feature = "record")]
    pub fn for_recorder(width: u32, height: u32) -> Self {
        let scale_factor = 1.0;

        let physical_size = PhysicalSize::new(width, height);
        let logical_size = physical_size.to_logical::<f32>(scale_factor);

        Self {
            physical_size,
            logical_size,

            scale_factor,

            cursor_physical_position: Default::default(),
            cursor_logical_position: Default::default(),

            focused: false,

            modifers_state: ModifiersState::default(),
        }
    }

    pub fn new(window: &winit::window::Window) -> Self {
        let scale_factor = window.scale_factor();

        #[cfg(not(feature = "record"))]
        let (physical_size, logical_size) = {
            let physical_size = window.inner_size();
            let logical_size = physical_size.to_logical::<f32>(scale_factor);
            (physical_size, logical_size)
        };
        #[cfg(feature = "record")]
        let (physical_size, logical_size) = {
            let physical_size = PhysicalSize::new(1920, 1080);
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

            modifers_state: ModifiersState::default(),
        }
    }

    pub fn window_event(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::Resized(ps) => {
                self.physical_size = *ps;
                self.logical_size = ps.to_logical(self.scale_factor);
            }
            WindowEvent::ScaleFactorChanged {
                scale_factor,
                new_inner_size,
            } => {
                self.physical_size = **new_inner_size;
                self.logical_size = new_inner_size.to_logical(self.scale_factor);

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
                self.modifers_state = *state;
            }
            _ => {}
        }
    }
}
