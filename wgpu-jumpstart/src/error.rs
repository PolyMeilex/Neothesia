#[derive(Debug)]
pub enum GpuInitError {
    AdapterRequest(wgpu::RequestAdapterError),
    DeviceRequest(wgpu::RequestDeviceError),
    CreateSurfaceError(wgpu::CreateSurfaceError),

    #[cfg(target_arch = "wasm32")]
    AppendToBody,
}

impl From<wgpu::CreateSurfaceError> for GpuInitError {
    fn from(value: wgpu::CreateSurfaceError) -> Self {
        Self::CreateSurfaceError(value)
    }
}

impl From<wgpu::RequestAdapterError> for GpuInitError {
    fn from(value: wgpu::RequestAdapterError) -> Self {
        Self::AdapterRequest(value)
    }
}

impl std::fmt::Display for GpuInitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use GpuInitError::*;
        match self {
            AdapterRequest(err) => write!(f, "Failed to create adapter: {err}"),
            #[cfg(target_arch = "wasm32")]
            AppendToBody => write!(f, "Couldn't append canvas to document body"),
            _ => write!(f, "{self:?}"),
        }
    }
}

impl std::error::Error for GpuInitError {}
