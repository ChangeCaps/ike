use crate::{render_device, render_queue};

use ike_wgpu as wgpu;

pub struct Buffer {
    len: u64,
    raw_buffer: Option<wgpu::Buffer>,
    usage: wgpu::BufferUsages,
}

impl Clone for Buffer {
    #[inline]
    fn clone(&self) -> Self {
        let raw_buffer = self.raw_buffer.as_ref().map(|source| {
            let buffer = render_device().create_buffer(&wgpu::BufferDescriptor {
                label: None,
                size: self.len,
                usage: self.usage,
                mapped_at_creation: false,
            });

            let mut encoder = render_device().create_command_encoder(&Default::default());

            encoder.copy_buffer_to_buffer(source, 0, &buffer, 0, self.len);

            render_queue().submit_once(encoder.finish());

            buffer
        });

        Self {
            len: self.len,
            raw_buffer,
            usage: self.usage,
        }
    }
}

impl Buffer {
    #[inline]
    pub fn new(mut usage: wgpu::BufferUsages) -> Self {
        usage |= wgpu::BufferUsages::COPY_DST;

        Self {
            len: 0,
            raw_buffer: None,
            usage,
        }
    }

    #[inline]
    pub fn write(&mut self, data: &[u8]) {
        if self.len < data.len() as u64 || self.raw_buffer.is_none() {
            let buffer = render_device().create_buffer_init(&wgpu::BufferInitDescriptor {
                label: None,
                contents: data,
                usage: self.usage,
            });

            self.raw_buffer = Some(buffer);
            self.len = data.len() as u64;
        } else {
            let raw_buffer = self.raw_buffer.as_ref().unwrap();

            render_queue().write_buffer(raw_buffer, 0, data);
        }
    }

    #[inline]
    pub fn raw(&mut self) -> &wgpu::Buffer {
        if self.raw_buffer.is_none() {
            self.write(&[]);
        }

        self.raw_buffer.as_ref().unwrap()
    }

    #[inline]
    pub fn get_raw(&self) -> Option<&wgpu::Buffer> {
        self.raw_buffer.as_ref()
    }
}
