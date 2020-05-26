use super::surface::Surface;

pub struct Gpu {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub encoder: wgpu::CommandEncoder,
}

impl Gpu {
    pub async fn for_window(window: &winit::window::Window) -> (Self, Surface) {
        let instance = wgpu::Instance::new();
        let surface = unsafe { instance.create_surface(window) };

        let adapter = instance
            .request_adapter(
                &wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::HighPerformance,
                    compatible_surface: Some(&surface),
                },
                wgpu::BackendBit::PRIMARY,
            )
            .await
            .expect("Failed to create adapter");

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    extensions: wgpu::Extensions {
                        anisotropic_filtering: false,
                    },
                    limits: Default::default(),
                },
                None,
            )
            .await
            .unwrap();

        let surface = Surface::new(window, surface, &device);

        let encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        (
            Self {
                device,
                queue,
                encoder,
            },
            surface,
        )
    }
    pub fn submit(&mut self) {
        let new_encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        // We swap the current decoder by a new one here, so we can finish the
        // current frame
        let encoder = std::mem::replace(&mut self.encoder, new_encoder);

        self.queue.submit(Some(encoder.finish()));
    }
}
