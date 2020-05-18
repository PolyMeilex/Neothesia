use zerocopy::AsBytes;

pub struct Uniform<U>
where
    U: 'static + Copy + AsBytes,
{
    pub data: U,
    buffer: wgpu::Buffer,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
}
impl<U> Uniform<U>
where
    U: 'static + Copy + AsBytes,
{
    pub fn new(device: &wgpu::Device, data: U, visibility: wgpu::ShaderStage) -> Self {
        let bind_group_layout_descriptor = wgpu::BindGroupLayoutDescriptor {
            label: None,
            bindings: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility,
                ty: wgpu::BindingType::UniformBuffer { dynamic: false },
            }],
        };

        let buffer = device.create_buffer_with_data(
            &data.as_bytes(),
            wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        );

        let bind_group_layout = device.create_bind_group_layout(&bind_group_layout_descriptor);

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            bindings: &[wgpu::Binding {
                binding: 0,
                resource: wgpu::BindingResource::Buffer {
                    buffer: &buffer,
                    range: 0..std::mem::size_of_val(&data) as wgpu::BufferAddress,
                },
            }],
        });

        Self {
            data,
            bind_group,
            bind_group_layout,
            buffer,
        }
    }
    pub fn update(&self, command_encoder: &mut wgpu::CommandEncoder, device: &wgpu::Device) {
        let staging_buffer =
            device.create_buffer_with_data(&self.data.as_bytes(), wgpu::BufferUsage::COPY_SRC);
        command_encoder.copy_buffer_to_buffer(
            &staging_buffer,
            0,
            &self.buffer,
            0,
            std::mem::size_of::<U>() as wgpu::BufferAddress,
        );
    }
}
