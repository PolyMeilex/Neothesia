pub fn create_module(device: &wgpu::Device, spv: &[u8]) -> wgpu::ShaderModule {
    let foo = wgpu::read_spirv(std::io::Cursor::new(&spv[..])).unwrap();
    device.create_shader_module(&foo)
}
