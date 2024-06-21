use super::color::Color;
use super::GpuInitError;

pub fn default_backends() -> wgpu::Backends {
    wgpu::util::backend_bits_from_env().unwrap_or(wgpu::Backends::all())
}

pub struct Gpu {
    pub device: wgpu::Device,

    pub adapter: wgpu::Adapter,
    pub queue: wgpu::Queue,
    pub encoder: wgpu::CommandEncoder,
    pub texture_format: wgpu::TextureFormat,
}

impl Gpu {
    pub async fn for_window(
        window: impl Into<wgpu::SurfaceTarget<'static>>,
        width: u32,
        height: u32,
    ) -> Result<(Self, Surface), GpuInitError> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: crate::default_backends(),
            dx12_shader_compiler: wgpu::Dx12Compiler::default(),
            flags: wgpu::InstanceFlags::default(),
            gles_minor_version: wgpu::Gles3MinorVersion::Automatic,
        });

        let surface = instance.create_surface(window.into())?;
        let gpu = Self::new(&instance, Some(&surface)).await?;
        let surface = Surface::new(&gpu.device, surface, gpu.texture_format, width, height);

        Ok((gpu, surface))
    }

    pub async fn new(
        instance: &wgpu::Instance,
        compatible_surface: Option<&wgpu::Surface<'static>>,
    ) -> Result<Self, GpuInitError> {
        let power_preference = wgpu::util::power_preference_from_env()
            .unwrap_or(wgpu::PowerPreference::HighPerformance);

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference,
                compatible_surface,
                force_fallback_adapter: false,
            })
            .await
            .ok_or(GpuInitError::AdapterRequest)?;

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits {
                        max_compute_workgroup_storage_size: 0,
                        max_compute_invocations_per_workgroup: 0,
                        max_compute_workgroup_size_x: 0,
                        max_compute_workgroup_size_y: 0,
                        max_compute_workgroup_size_z: 0,
                        max_compute_workgroups_per_dimension: 0,
                        ..wgpu::Limits::downlevel_defaults()
                    }
                    .using_resolution(adapter.limits()),
                    ..Default::default()
                },
                None,
            )
            .await
            .map_err(GpuInitError::DeviceRequest)?;

        let encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        let adapter_info = adapter.get_info();
        let texture_format = compatible_surface.map(|s| s.get_capabilities(&adapter).formats[0]);

        log::info!(
            "Using {} ({:?}, Preferred Format: {:?})",
            adapter_info.name,
            adapter_info.backend,
            texture_format,
        );

        Ok(Self {
            device,
            adapter,
            queue,
            encoder,
            texture_format: texture_format.unwrap_or(wgpu::TextureFormat::Bgra8UnormSrgb),
        })
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
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
    }

    pub fn take(&mut self) -> wgpu::CommandEncoder {
        let new_encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        // We swap the current decoder by a new one here, so we can finish the
        // current frame
        std::mem::replace(&mut self.encoder, new_encoder)
    }

    pub fn submit(&mut self) {
        let encoder = self.take();
        self.queue.submit(Some(encoder.finish()));
    }
}

pub struct Surface {
    surface: wgpu::Surface<'static>,
    surface_configuration: wgpu::SurfaceConfiguration,
}

impl Surface {
    pub fn new(
        device: &wgpu::Device,
        surface: wgpu::Surface<'static>,
        texture_format: wgpu::TextureFormat,
        width: u32,
        height: u32,
    ) -> Self {
        let surface_configuration = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: texture_format,
            view_formats: vec![texture_format],
            width,
            height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            desired_maximum_frame_latency: 2,
        };

        surface.configure(device, &surface_configuration);

        Self {
            surface,
            surface_configuration,
        }
    }

    #[inline]
    pub fn get_current_texture(&mut self) -> Result<wgpu::SurfaceTexture, wgpu::SurfaceError> {
        self.surface.get_current_texture()
    }

    pub fn resize_swap_chain(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        self.surface_configuration.width = width;
        self.surface_configuration.height = height;

        self.surface.configure(device, &self.surface_configuration);
    }
}
