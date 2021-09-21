pub(crate) unsafe trait CommandEncoderTrait {
    fn finish(self: Box<Self>) -> crate::CommandBuffer;

    fn begin_render_pass<'a>(
        &'a mut self,
        desc: &crate::RenderPassDescriptor<'a, '_>,
    ) -> crate::RenderPass<'a>;

    fn begin_compute_pass(
        &mut self,
        desc: &crate::ComputePassDescriptor<'_>,
    ) -> crate::ComputePass<'_>;

    fn copy_texture_to_buffer(
        &mut self,
        source: crate::ImageCopyTexture<&crate::Texture>,
        destination: crate::ImageCopyBuffer<&crate::Buffer>,
        copy_size: crate::Extent3d,
    );
}

#[cfg(feature = "wgpu")]
unsafe impl CommandEncoderTrait for wgpu::CommandEncoder {
    #[inline]
    fn finish(self: Box<Self>) -> crate::CommandBuffer {
        crate::CommandBuffer(Box::new((*self).finish()))
    }

    #[inline]
    fn begin_render_pass<'a>(
        &'a mut self,
        desc: &crate::RenderPassDescriptor<'a, '_>,
    ) -> crate::RenderPass<'a> {
        fn convert_ops<V>(ops: crate::Operations<V>) -> wgpu::Operations<V> {
            wgpu::Operations {
                load: match ops.load {
                    crate::LoadOp::Clear(v) => wgpu::LoadOp::Clear(v),
                    crate::LoadOp::Load => wgpu::LoadOp::Load,
                },
                store: ops.store,
            }
        }

        let color_attachments = desc
            .color_attachments
            .iter()
            .map(|color| wgpu::RenderPassColorAttachment {
                view: unsafe { &*(color.view.0.as_ref() as *const _ as *const _) },
                resolve_target: color
                    .resolve_target
                    .map(|target| unsafe { &*(target.0.as_ref() as *const _ as *const _) }),
                ops: convert_ops(color.ops.clone()),
            })
            .collect::<Vec<_>>();

        let depth_stencil_attachment = desc.depth_stencil_attachment.as_ref().map(|depth| {
            wgpu::RenderPassDepthStencilAttachment {
                view: unsafe { &*(depth.view.0.as_ref() as *const _ as *const _) },
                depth_ops: depth.depth_ops.clone().map(convert_ops),
                stencil_ops: depth.stencil_ops.clone().map(convert_ops),
            }
        });

        let desc = wgpu::RenderPassDescriptor {
            label: desc.label,
            color_attachments: &color_attachments,
            depth_stencil_attachment,
        };

        crate::RenderPass(Box::new(self.begin_render_pass(&desc)))
    }

    #[inline]
    fn begin_compute_pass(
        &mut self,
        desc: &crate::ComputePassDescriptor<'_>,
    ) -> crate::ComputePass<'_> {
        let pass = self.begin_compute_pass(&wgpu::ComputePassDescriptor { label: desc.label });

        crate::ComputePass(Box::new(pass))
    }

    #[inline]
    fn copy_texture_to_buffer(
        &mut self,
        source: crate::ImageCopyTexture<&crate::Texture>,
        destination: crate::ImageCopyBuffer<&crate::Buffer>,
        copy_size: crate::Extent3d,
    ) {
        self.copy_texture_to_buffer(
            wgpu::ImageCopyTexture {
                texture: unsafe { &*(source.texture.0.as_ref() as *const _ as *const _) },
                mip_level: source.mip_level,
                origin: source.origin,
                aspect: source.aspect,
            },
            wgpu::ImageCopyBuffer {
                buffer: unsafe { &*(destination.buffer.0.as_ref() as *const _ as *const _) },
                layout: destination.layout,
            },
            copy_size,
        );
    }
}

pub struct CommandEncoder(pub(crate) Box<dyn CommandEncoderTrait>);

impl CommandEncoder {
    #[inline]
    pub fn finish(self) -> crate::CommandBuffer {
        self.0.finish()
    }

    #[inline]
    pub fn begin_render_pass<'a>(
        &'a mut self,
        desc: &crate::RenderPassDescriptor<'a, '_>,
    ) -> crate::RenderPass<'a> {
        self.0.begin_render_pass(desc)
    }

    #[inline]
    pub fn begin_compute_pass(
        &mut self,
        desc: &crate::ComputePassDescriptor<'_>,
    ) -> crate::ComputePass<'_> {
        self.0.begin_compute_pass(desc)
    }

    #[inline]
    pub fn copy_texture_to_buffer(
        &mut self,
        source: crate::ImageCopyTexture<&crate::Texture>,
        destination: crate::ImageCopyBuffer<&crate::Buffer>,
        copy_size: crate::Extent3d,
    ) {
        self.0
            .copy_texture_to_buffer(source, destination, copy_size);
    }
}

pub(crate) unsafe trait CommandBufferTrait {}

#[cfg(feature = "wgpu")]
unsafe impl CommandBufferTrait for wgpu::CommandBuffer {}

pub struct CommandBuffer(pub(crate) Box<dyn CommandBufferTrait>);
