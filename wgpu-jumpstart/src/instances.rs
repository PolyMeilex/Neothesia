use bytemuck::Pod;

pub struct Instances<I>
where
    I: Pod,
{
    pub data: Vec<I>,
    pub buffer: wgpu::Buffer,
}
impl<I> Instances<I>
where
    I: Pod,
{
    pub fn new(device: &wgpu::Device, max_size: usize) -> Self {
        let instance_size = std::mem::size_of::<I>();
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: (instance_size * max_size) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            data: Vec::new(),
            buffer,
        }
    }

    pub fn update(&self, queue: &wgpu::Queue) {
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&self.data));
    }

    pub fn len(&self) -> u32 {
        self.data.len() as u32
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}
