#[derive(Clone, Debug)]
pub struct ComputePipelineDescriptor<'a> {
    pub label: Option<&'a str>,
    pub layout: Option<&'a crate::PipelineLayout>,
    pub module: &'a crate::ShaderModule,
    pub entry_point: &'a str,
}

pub(crate) unsafe trait ComputePipelineTrait {}

#[cfg(feature = "wgpu")]
unsafe impl ComputePipelineTrait for wgpu::ComputePipeline {}

pub struct ComputePipeline(pub(crate) Box<dyn ComputePipelineTrait>);

#[derive(Default, Clone, Debug)]
pub struct ComputePassDescriptor<'a> {
    pub label: Option<&'a str>,
}

pub(crate) unsafe trait ComputePassTrait<'a> {
    fn set_bind_group(&mut self, index: u32, bind_group: &'a crate::BindGroup, offsets: &[u32]);

    fn set_pipeline(&mut self, pipeline: &'a crate::ComputePipeline);

    fn dispatch(&mut self, x: u32, y: u32, z: u32);
}

#[cfg(feature = "wgpu")]
unsafe impl<'a> ComputePassTrait<'a> for wgpu::ComputePass<'a> {
    #[inline]
    fn set_bind_group(&mut self, index: u32, bind_group: &'a crate::BindGroup, offsets: &[u32]) {
        self.set_bind_group(
            index,
            unsafe { &*(bind_group.0.as_ref() as *const _ as *const _) },
            offsets,
        );
    }

    #[inline]
    fn set_pipeline(&mut self, pipeline: &'a crate::ComputePipeline) {
        self.set_pipeline(unsafe { &*(pipeline.0.as_ref() as *const _ as *const _) });
    }

    #[inline]
    fn dispatch(&mut self, x: u32, y: u32, z: u32) {
        self.dispatch(x, y, z);
    }
}

pub struct ComputePass<'a>(pub(crate) Box<dyn ComputePassTrait<'a> + 'a>);

impl<'a> ComputePass<'a> {
    #[inline]
    pub fn set_bind_group(
        &mut self,
        index: u32,
        bind_group: &'a crate::BindGroup,
        offsets: &[u32],
    ) {
        self.0.set_bind_group(index, bind_group, offsets);
    }

    #[inline]
    pub fn set_pipeline(&mut self, pipeline: &'a crate::ComputePipeline) {
        self.0.set_pipeline(pipeline);
    }

    #[inline]
    pub fn dispatch(&mut self, x: u32, y: u32, z: u32) {
        self.0.dispatch(x, y, z);
    }
}
