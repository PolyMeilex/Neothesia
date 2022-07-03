use super::color::Color;
use super::GpuInitError;

pub struct Gpu {
    pub device: wgpu::Device,

    pub queue: wgpu::Queue,
    pub encoder: wgpu::CommandEncoder,
    pub staging_belt: wgpu::util::StagingBelt,

    pub backend: wgpu::Backend,
}

impl Gpu {
    pub async fn for_window(
        window: &winit::window::Window,
    ) -> Result<(Self, wgpu::Surface), GpuInitError> {
        let backend = wgpu::util::backend_bits_from_env().unwrap_or(wgpu::Backends::all());
        let power_preference = wgpu::util::power_preference_from_env()
            .unwrap_or(wgpu::PowerPreference::HighPerformance);

        let instance = wgpu::Instance::new(backend);

        let surface = unsafe { instance.create_surface(window) };

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .ok_or(GpuInitError::AdapterRequest)?;

        let adapter_info = adapter.get_info();
        let format = surface.get_supported_formats(&adapter)[0];

        log::info!(
            "Using {} ({:?}, Preferred Format: {:?})",
            adapter_info.name,
            adapter_info.backend,
            format
        );

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default(), None)
            .await
            .map_err(GpuInitError::DeviceRequest)?;

        let encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        let staging_belt = wgpu::util::StagingBelt::new(5 * 1024);

        Ok((
            Self {
                device,
                queue,
                encoder,
                staging_belt,

                backend: adapter_info.backend,
            },
            surface,
        ))
    }

    pub fn clear(&mut self, view: &wgpu::TextureView, color: Color) {
        let rgb = color.into_linear_rgb();
        self.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: rgb[0] as f64,
                        g: rgb[1] as f64,
                        b: rgb[2] as f64,
                        a: 1.0,
                    }),
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        });
    }

    pub fn submit(&mut self) -> Result<(), futures::task::SpawnError> {
        let new_encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        // We swap the current decoder by a new one here, so we can finish the
        // current frame
        let encoder = std::mem::replace(&mut self.encoder, new_encoder);

        self.staging_belt.finish();
        self.queue.submit(Some(encoder.finish()));

        self.staging_belt.recall();

        Ok(())
    }
}
