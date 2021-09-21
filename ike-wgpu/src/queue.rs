pub(crate) unsafe trait QueueTrait: Send + Sync {
    fn submit(&self, command_buffers: Vec<crate::CommandBuffer>);

    fn write_buffer(&self, buffer: &crate::Buffer, offset: u64, data: &[u8]);

    fn write_texture(
        &self,
        texture: crate::ImageCopyTexture<&crate::Texture>,
        data: &[u8],
        data_layout: crate::ImageDataLayout,
        size: crate::Extent3d,
    );
}

#[cfg(feature = "wgpu")]
unsafe impl QueueTrait for wgpu::Queue {
    #[inline]
    fn submit(&self, command_buffers: Vec<crate::CommandBuffer>) {
        let command_buffers = command_buffers
            .into_iter()
            .map(|buffer| unsafe { *Box::from_raw(Box::into_raw(buffer.0) as *mut _) });

        self.submit(command_buffers)
    }

    #[inline]
    fn write_buffer(&self, buffer: &crate::Buffer, offset: u64, data: &[u8]) {
        self.write_buffer(
            unsafe { &*(buffer.0.as_ref() as *const _ as *const _) },
            offset,
            data,
        );
    }

    #[inline]
    fn write_texture(
        &self,
        texture: crate::ImageCopyTexture<&crate::Texture>,
        data: &[u8],
        data_layout: crate::ImageDataLayout,
        size: crate::Extent3d,
    ) {
        self.write_texture(
            wgpu::ImageCopyTexture {
                texture: unsafe { &*(texture.texture.0.as_ref() as *const _ as *const _) },
                mip_level: texture.mip_level,
                origin: texture.origin,
                aspect: texture.aspect,
            },
            data,
            data_layout,
            size,
        );
    }
}

pub struct Queue(pub(crate) Box<dyn QueueTrait>);

impl Queue {
    #[cfg(feature = "wgpu")]
    #[inline]
    pub fn new(queue: wgpu::Queue) -> Self {
        Self(Box::new(queue))
    }

    #[inline]
    pub fn submit(&self, command_buffers: impl Iterator<Item = crate::CommandBuffer>) {
        self.0.submit(command_buffers.collect());
    }

    #[inline]
    pub fn submit_once(&self, command_buffer: crate::CommandBuffer) {
        self.0.submit(vec![command_buffer]);
    }

    #[inline]
    pub fn write_buffer(&self, buffer: &crate::Buffer, offset: u64, data: &[u8]) {
        self.0.write_buffer(buffer, offset, data);
    }

    #[inline]
    pub fn write_texture(
        &self,
        texture: crate::ImageCopyTexture<&crate::Texture>,
        data: &[u8],
        data_layout: crate::ImageDataLayout,
        size: crate::Extent3d,
    ) {
        self.0.write_texture(texture, data, data_layout, size);
    }
}
