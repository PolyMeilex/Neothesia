use super::{GpuInitError, color::Color};

pub struct Gpu {
    pub device: wgpu::Device,

    pub adapter: wgpu::Adapter,
    pub queue: wgpu::Queue,
    pub encoder: wgpu::CommandEncoder,
    pub texture_format: wgpu::TextureFormat,
}

impl Gpu {
    async fn try_init(
        window: wgpu::SurfaceTarget<'static>,
        desc: &wgpu::InstanceDescriptor,
    ) -> Result<(Self, wgpu::Surface<'static>), GpuInitError> {
        log::info!("Trying to initialize GPU with: {:?}", desc.backends);

        let instance = wgpu::Instance::new(desc);
        let surface = instance.create_surface(window)?;

        Ok((Self::new(&instance, Some(&surface)).await?, surface))
    }

    pub async fn for_window(
        window: impl Fn() -> wgpu::SurfaceTarget<'static>,
        width: u32,
        height: u32,
    ) -> Result<(Self, Surface), GpuInitError> {
        enum FallbackStateMachine {
            DefaultOrEnv,
            Dx12,
            Vulkan,
            Gl,
        }

        let mut state_machine = FallbackStateMachine::DefaultOrEnv;
        let mut desc = wgpu::InstanceDescriptor::from_env_or_default();

        let (gpu, surface) = loop {
            let res = Self::try_init(window(), &desc).await;

            match res {
                Ok(res) => break res,
                Err(err) if cfg!(target_os = "macos") => return Err(err),
                // Wgpu backend picking leaves much to be desired, so let's bruteforce all possible
                // backend options manually before giving up
                Err(err) => match state_machine {
                    FallbackStateMachine::DefaultOrEnv => {
                        if cfg!(target_os = "windows") {
                            log::error!("'{err}': fallbacking to DX12");
                            desc.backends = wgpu::Backends::DX12;
                            state_machine = FallbackStateMachine::Dx12;
                        } else {
                            log::error!("'{err}': fallbacking to Vulkan");
                            desc.backends = wgpu::Backends::VULKAN;
                            state_machine = FallbackStateMachine::Vulkan;
                        }
                    }
                    FallbackStateMachine::Dx12 => {
                        log::error!("'{err}': fallbacking to Vulkan");
                        desc.backends = wgpu::Backends::VULKAN;
                        state_machine = FallbackStateMachine::Vulkan;
                    }
                    FallbackStateMachine::Vulkan => {
                        log::error!("'{err}': fallbacking to OpenGl");
                        desc.backends = wgpu::Backends::GL;
                        state_machine = FallbackStateMachine::Gl;
                    }
                    FallbackStateMachine::Gl => return Err(err),
                },
            }
        };

        let surface = Surface::new(&gpu.device, surface, gpu.texture_format, width, height);

        Ok((gpu, surface))
    }

    pub async fn new(
        instance: &wgpu::Instance,
        compatible_surface: Option<&wgpu::Surface<'static>>,
    ) -> Result<Self, GpuInitError> {
        let power_preference =
            wgpu::PowerPreference::from_env().unwrap_or(wgpu::PowerPreference::HighPerformance);

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference,
                compatible_surface,
                force_fallback_adapter: false,
            })
            .await?;

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
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
            })
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
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
    }

    fn take_encoder(&mut self) -> wgpu::CommandEncoder {
        let new_encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        // We swap the current decoder by a new one here, so we can finish the
        // current frame
        std::mem::replace(&mut self.encoder, new_encoder)
    }

    pub fn submit(&mut self) {
        let encoder = self.take_encoder();
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
