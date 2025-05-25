use bytemuck::Pod;

pub struct Instances<I>
where
    I: Pod,
{
    pub data: Vec<I>,
    pub buffer: wgpu::Buffer,
    capacity: usize,
}
impl<I> Instances<I>
where
    I: Pod,
{
    pub fn new(device: &wgpu::Device, size_hint: usize) -> Self {
        Self {
            data: Vec::new(),
            buffer: Self::create_buffer(device, size_hint),
            capacity: size_hint,
        }
    }

    fn create_buffer(device: &wgpu::Device, len: usize) -> wgpu::Buffer {
        let instance_size = std::mem::size_of::<I>();
        device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: (instance_size * len) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    }

    pub fn update(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        if self.capacity < self.data.len() {
            log::trace!(
                "Dynamically growing instances buffer from {} to {}",
                self.capacity,
                self.data.len()
            );
            self.buffer = Self::create_buffer(device, self.data.len());
            self.capacity = self.data.len();
        }
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&self.data));
    }

    pub fn len(&self) -> u32 {
        self.data.len() as u32
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}
