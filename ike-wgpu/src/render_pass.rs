use std::ops::Range;

#[derive(Clone)]
pub enum LoadOp<V> {
    Clear(V),
    Load,
}

#[derive(Clone)]
pub struct Operations<V> {
    pub load: LoadOp<V>,
    pub store: bool,
}

pub struct RenderPassColorAttachment<'a> {
    pub view: &'a crate::TextureView,
    pub resolve_target: Option<&'a crate::TextureView>,
    pub ops: Operations<crate::Color>,
}

pub struct RenderPassDepthStencilAttachment<'a> {
    pub view: &'a crate::TextureView,
    pub depth_ops: Option<Operations<f32>>,
    pub stencil_ops: Option<Operations<u32>>,
}

pub struct RenderPassDescriptor<'a, 'b> {
    pub label: Option<&'a str>,
    pub color_attachments: &'b [RenderPassColorAttachment<'a>],
    pub depth_stencil_attachment: Option<RenderPassDepthStencilAttachment<'a>>,
}

pub(crate) unsafe trait RenderPassTrait<'a> {
    fn set_bind_group(&mut self, index: u32, bind_group: &'a crate::BindGroup, offsets: &[u32]);

    fn set_pipeline(&mut self, pipeline: &'a crate::RenderPipeline);

    fn set_index_buffer(
        &mut self,
        buffer_slice: crate::BufferSlice<'a>,
        index_format: crate::IndexFormat,
    );

    fn set_vertex_buffer(&mut self, slot: u32, buffer_slice: crate::BufferSlice<'a>);

    fn set_scissor_rect(&mut self, x: u32, y: u32, width: u32, height: u32);

    fn draw(&mut self, vertices: Range<u32>, instances: Range<u32>);
    
    fn draw_indexed(&mut self, indices: Range<u32>, base_vertex: i32, instances: Range<u32>);
}

#[cfg(feature = "wgpu")]
unsafe impl<'a> RenderPassTrait<'a> for wgpu::RenderPass<'a> {
    #[inline]
    fn set_bind_group(&mut self, index: u32, bind_group: &'a crate::BindGroup, offsets: &[u32]) {
        self.set_bind_group(
            index,
            unsafe { &*(bind_group.0.as_ref() as *const _ as *const _) },
            offsets,
        );
    }

    #[inline]
    fn set_pipeline(&mut self, pipeline: &'a crate::RenderPipeline) {
        self.set_pipeline(unsafe { &*(pipeline.0.as_ref() as *const _ as *const _) });
    }

    #[inline]
    fn set_index_buffer(
        &mut self,
        buffer_slice: crate::BufferSlice<'a>,
        index_format: crate::IndexFormat,
    ) {
        self.set_index_buffer(
            unsafe { *(buffer_slice.0.as_ref() as *const _ as *const _) },
            index_format,
        ); 
    }

    #[inline]
    fn set_vertex_buffer(&mut self, slot: u32, buffer_slice: crate::BufferSlice<'a>) {
        self.set_vertex_buffer(slot, unsafe { *(buffer_slice.0.as_ref() as *const _ as *const _) });
    }

    #[inline]
    fn set_scissor_rect(&mut self, x: u32, y: u32, width: u32, height: u32) {
        self.set_scissor_rect(x, y, width, height);
    }

    #[inline]
    fn draw(&mut self, vertices: Range<u32>, instances: Range<u32>) {
        self.draw(vertices, instances);
    }

    #[inline]
    fn draw_indexed(&mut self, indices: Range<u32>, base_vertex: i32, instances: Range<u32>) {
        self.draw_indexed(indices, base_vertex, instances);
    }
}

pub struct RenderPass<'a>(pub(crate) Box<dyn RenderPassTrait<'a> + 'a>);

impl<'a> RenderPass<'a> {
    #[inline]
    pub fn set_bind_group(&mut self, index: u32, bind_group: &'a crate::BindGroup, offsets: &[u32]) {
        self.0.set_bind_group(index, bind_group, offsets);
    }

    #[inline]
    pub fn set_pipeline(&mut self, pipeline: &'a crate::RenderPipeline) {
        self.0.set_pipeline(pipeline);
    }

    #[inline]
    pub fn set_index_buffer(
        &mut self,
        buffer_slice: crate::BufferSlice<'a>,
        index_format: crate::IndexFormat,
    ) {
        self.0.set_index_buffer(buffer_slice, index_format);
    }

    #[inline]
    pub fn set_vertex_buffer(&mut self, slot: u32, buffer_slice: crate::BufferSlice<'a>) {
        self.0.set_vertex_buffer(slot, buffer_slice);
    }

    #[inline]
    pub fn set_scissor_rect(&mut self, x: u32, y: u32, width: u32, height: u32) {
        self.0.set_scissor_rect(x, y, width, height);
    }

    #[inline]
    pub fn draw(&mut self, vertices: Range<u32>, instances: Range<u32>) {
        self.0.draw(vertices, instances);
    }

    #[inline]
    pub fn draw_indexed(&mut self, indices: Range<u32>, base_vertex: i32, instances: Range<u32>) {
        self.0.draw_indexed(indices, base_vertex, instances);
    }
}
