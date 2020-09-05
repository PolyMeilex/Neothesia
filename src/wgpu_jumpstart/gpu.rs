use super::surface::Surface;

pub struct Gpu {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub encoder: wgpu::CommandEncoder,
    pub staging_belt: wgpu::util::StagingBelt,
    pub local_pool: futures::executor::LocalPool,
}

impl Gpu {
    pub async fn for_window(window: &winit::window::Window) -> (Self, Surface) {
        let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);

        let surface = unsafe { instance.create_surface(window) };

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
            })
            .await
            .expect("Failed to create adapter");

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    limits: Default::default(),
                    shader_validation: false,
                },
                None,
            )
            .await
            .expect("Failed to request device");

        let surface = Surface::new(window, surface, &device);

        let encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        let staging_belt = wgpu::util::StagingBelt::new(5 * 1024);
        let local_pool = futures::executor::LocalPool::new();

        (
            Self {
                device,
                queue,
                encoder,
                staging_belt,
                local_pool,
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

        self.staging_belt.finish();
        self.queue.submit(Some(encoder.finish()));

        {
            use futures::task::SpawnExt;

            self.local_pool
                .spawner()
                .spawn(self.staging_belt.recall())
                .expect("Recall staging buffers");

            self.local_pool.run_until_stalled();
        }
    }
}
