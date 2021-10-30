use crate::renderer::{render_device, render_queue};

pub struct Buffer {
    len: usize,
    raw_buffer: Option<ike_wgpu::Buffer>,
    usage: ike_wgpu::BufferUsages,
}

impl Buffer {
    #[inline]
    pub fn new(mut usage: ike_wgpu::BufferUsages) -> Self {
        usage |= ike_wgpu::BufferUsages::COPY_DST;

        Self {
            len: 0,
            raw_buffer: None,
            usage,
        }
    }

    #[inline]
    pub fn write(&mut self, data: &[u8]) {
        if self.len < data.len() || self.raw_buffer.is_none() {
            let buffer = render_device().create_buffer_init(&ike_wgpu::BufferInitDescriptor {
                label: None,
                contents: data,
                usage: self.usage,
            });

            self.raw_buffer = Some(buffer);
            self.len = data.len();
        } else {
            let raw_buffer = self.raw_buffer.as_ref().unwrap();

            render_queue().write_buffer(raw_buffer, 0, data);
        }
    }

    #[inline]
    pub fn raw(&mut self) -> &ike_wgpu::Buffer {
        if self.raw_buffer.is_none() {
            self.write(&[]);
        }

        self.raw_buffer.as_ref().unwrap()
    }
}
