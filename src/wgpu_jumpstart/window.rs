use super::{Gpu, GpuInitError};
use std::collections::HashMap;
use winit::dpi::LogicalPosition;
use winit::dpi::PhysicalPosition;
use winit::event::ModifiersState;
use winit::event::MouseButton;

use winit::{
    dpi::{LogicalSize, PhysicalSize},
    event::{Event, WindowEvent},
};

pub struct Window {
    pub winit_window: winit::window::Window,
    pub state: WinitState,

    surface: wgpu::Surface,

    swap_chain: wgpu::SwapChain,
    swap_chain_descriptor: wgpu::SwapChainDescriptor,
}

impl Window {
    pub async fn new(winit_window: winit::window::Window) -> Result<(Self, Gpu), GpuInitError> {
        #[cfg(target_arch = "wasm32")]
        {
            use winit::platform::web::WindowExtWebSys;
            web_sys::window()
                .and_then(|win| win.document())
                .and_then(|doc| doc.body())
                .and_then(|body| {
                    body.append_child(&web_sys::Element::from(winit_window.canvas()))
                        .ok()
                })
                .unwrap_or(Err(GpuInitError::AppendToBody)?);
        }

        let (gpu, surface) = Gpu::for_window(&winit_window).await?;

        let (swap_chain, swap_chain_descriptor) = {
            #[cfg(not(feature = "record"))]
            let PhysicalSize { width, height } = winit_window.inner_size();
            #[cfg(feature = "record")]
            let (width, height) = { (1920, 1080) };

            let swap_chain_descriptor = wgpu::SwapChainDescriptor {
                usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
                format: super::TEXTURE_FORMAT,
                width: width,
                height: height,
                present_mode: wgpu::PresentMode::Fifo,
            };

            let swap_chain = gpu
                .device
                .create_swap_chain(&surface, &swap_chain_descriptor);
            (swap_chain, swap_chain_descriptor)
        };

        let state = WinitState::new(&winit_window);

        Ok((
            Self {
                winit_window,
                state,

                surface,

                swap_chain,
                swap_chain_descriptor,
            },
            gpu,
        ))
    }

    pub fn on_event<T>(&mut self, gpu: &mut Gpu, event: &Event<T>) {
        self.state.update(event);

        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::Resized(_) | WindowEvent::ScaleFactorChanged { .. } => {
                    self.resize_swap_chain(gpu)
                }
                _ => {}
            },
            _ => {}
        }
    }

    #[inline]
    pub fn request_redraw(&self) {
        self.winit_window.request_redraw();
    }

    #[inline]
    pub fn get_current_frame(&mut self) -> Result<wgpu::SwapChainFrame, wgpu::SwapChainError> {
        self.swap_chain.get_current_frame()
    }

    fn resize_swap_chain(&mut self, gpu: &mut Gpu) {
        let size = &self.state.physical_size;

        self.swap_chain_descriptor.width = size.width;
        self.swap_chain_descriptor.height = size.height;

        self.swap_chain = gpu
            .device
            .create_swap_chain(&self.surface, &self.swap_chain_descriptor);
    }
}

pub struct WinitState {
    pub physical_size: PhysicalSize<u32>,
    pub logical_size: LogicalSize<f32>,

    pub scale_factor: f64,

    pub cursor_physical_position: PhysicalPosition<f64>,
    pub cursor_logical_position: LogicalPosition<f32>,

    pub focused: bool,

    pub modifers_state: ModifiersState,

    /// Mouse Was Clicked This Frame
    mouse_clicked_events: Vec<MouseButton>,
    mouse_buttons_state: HashMap<MouseButton, bool>,
}

impl WinitState {
    fn new(window: &winit::window::Window) -> Self {
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

            mouse_clicked_events: Vec::new(),
            mouse_buttons_state: HashMap::new(),
        }
    }

    pub fn mouse_was_pressed(&self, button: MouseButton) -> bool {
        for btn in self.mouse_clicked_events.iter() {
            if &button == btn {
                return true;
            }
        }

        false
    }

    pub fn mouse_is_pressed(&self, button: MouseButton) -> bool {
        if let Some(s) = self.mouse_buttons_state.get(&button) {
            *s
        } else {
            false
        }
    }

    fn update<T>(&mut self, event: &Event<T>) {
        match event {
            Event::NewEvents { .. } => {
                self.mouse_clicked_events.clear();
            }
            Event::WindowEvent { event, .. } => match event {
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
                WindowEvent::MouseInput { state, button, .. } => {
                    if let winit::event::ElementState::Pressed = state {
                        self.mouse_clicked_events.push(*button);
                        self.mouse_buttons_state.insert(*button, true);
                    } else {
                        self.mouse_buttons_state.insert(*button, false);
                    }
                }
                WindowEvent::Focused(f) => {
                    self.focused = *f;
                    if f == &false {
                        self.mouse_buttons_state.clear();
                    }
                }
                WindowEvent::ModifiersChanged(state) => {
                    self.modifers_state = *state;
                }
                _ => {}
            },
            _ => {}
        }
    }
}
