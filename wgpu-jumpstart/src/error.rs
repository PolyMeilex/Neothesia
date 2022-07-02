#[derive(Debug)]
pub enum GpuInitError {
    AdapterRequest,
    DeviceRequest(wgpu::RequestDeviceError),

    #[cfg(target_arch = "wasm32")]
    AppendToBody,
}

impl std::fmt::Display for GpuInitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use GpuInitError::*;
        match self {
            AdapterRequest => write!(f, "Failed to create adapter"),
            #[cfg(target_arch = "wasm32")]
            AppendToBody => write!(f, "Couldn't append canvas to document body"),
            _ => write!(f, "{:?}", self),
        }
    }
}

impl std::error::Error for GpuInitError {}
