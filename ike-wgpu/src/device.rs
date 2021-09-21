#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Maintain {
    Wait,
    Poll,
}

pub(crate) unsafe trait DeviceTrait: 'static {
    fn poll(&self, maintain: Maintain);

    fn create_buffer(&self, desc: &crate::BufferDescriptor<Option<&'static str>>) -> crate::Buffer;

    fn create_buffer_init(&self, desc: &crate::BufferInitDescriptor) -> crate::Buffer;

    fn create_texture(
        &self,
        desc: &crate::TextureDescriptor<Option<&'static str>>,
    ) -> crate::Texture;

    fn create_texture_with_data(
        &self,
        queue: &crate::Queue,
        desc: &crate::TextureDescriptor<Option<&'static str>>,
        data: &[u8],
    ) -> crate::Texture;

    fn create_sampler(&self, desc: &crate::SamplerDescriptor) -> crate::Sampler;

    fn create_shader_module(&self, desc: &crate::ShaderModuleDescriptor) -> crate::ShaderModule;

    fn create_bind_group_layout(
        &self,
        desc: &crate::BindGroupLayoutDescriptor,
    ) -> crate::BindGroupLayout;

    fn create_bind_group(&self, desc: &crate::BindGroupDescriptor) -> crate::BindGroup;

    fn create_pipeline_layout(
        &self,
        desc: &crate::PipelineLayoutDescriptor,
    ) -> crate::PipelineLayout;

    fn create_render_pipeline(
        &self,
        desc: &crate::RenderPipelineDescriptor,
    ) -> crate::RenderPipeline;

    fn create_compute_pipeline(
        &self,
        desc: &crate::ComputePipelineDescriptor,
    ) -> crate::ComputePipeline;

    fn create_command_encoder(
        &self,
        desc: &crate::CommandEncoderDescriptor<Option<&'static str>>,
    ) -> crate::CommandEncoder;
}

pub struct Device(pub(crate) Box<dyn DeviceTrait>);

impl Device {
    #[inline]
    #[cfg(feature = "wgpu")]
    pub fn new(device: wgpu::Device) -> Self {
        Self(Box::new(device))
    }

    #[inline]
    pub fn poll(&self, maintain: Maintain) {
        self.0.poll(maintain);
    }

    #[inline]
    pub fn create_buffer(
        &self,
        desc: &crate::BufferDescriptor<Option<&'static str>>,
    ) -> crate::Buffer {
        self.0.create_buffer(desc)
    }

    #[inline]
    pub fn create_buffer_init(&self, desc: &crate::BufferInitDescriptor) -> crate::Buffer {
        self.0.create_buffer_init(desc)
    }

    #[inline]
    pub fn create_texture(
        &self,
        desc: &crate::TextureDescriptor<Option<&'static str>>,
    ) -> crate::Texture {
        self.0.create_texture(desc)
    }

    #[inline]
    pub fn create_texture_with_data(
        &self,
        queue: &crate::Queue,
        desc: &crate::TextureDescriptor<Option<&'static str>>,
        data: &[u8],
    ) -> crate::Texture {
        self.0.create_texture_with_data(queue, desc, data)
    }

    #[inline]
    pub fn create_sampler(&self, desc: &crate::SamplerDescriptor) -> crate::Sampler {
        self.0.create_sampler(desc)
    }

    #[inline]
    pub fn create_shader_module(
        &self,
        desc: &crate::ShaderModuleDescriptor,
    ) -> crate::ShaderModule {
        self.0.create_shader_module(desc)
    }

    #[inline]
    pub fn create_bind_group_layout(
        &self,
        desc: &crate::BindGroupLayoutDescriptor,
    ) -> crate::BindGroupLayout {
        self.0.create_bind_group_layout(desc)
    }

    #[inline]
    pub fn create_bind_group(&self, desc: &crate::BindGroupDescriptor) -> crate::BindGroup {
        self.0.create_bind_group(desc)
    }

    #[inline]
    pub fn create_pipeline_layout(
        &self,
        desc: &crate::PipelineLayoutDescriptor,
    ) -> crate::PipelineLayout {
        self.0.create_pipeline_layout(desc)
    }

    #[inline]
    pub fn create_render_pipeline(
        &self,
        desc: &crate::RenderPipelineDescriptor,
    ) -> crate::RenderPipeline {
        self.0.create_render_pipeline(desc)
    }

    #[inline]
    pub fn create_compute_pipeline(
        &self,
        desc: &crate::ComputePipelineDescriptor,
    ) -> crate::ComputePipeline {
        self.0.create_compute_pipeline(desc)
    }

    #[inline]
    pub fn create_command_encoder(
        &self,
        desc: &crate::CommandEncoderDescriptor<Option<&'static str>>,
    ) -> crate::CommandEncoder {
        self.0.create_command_encoder(desc)
    }
}

#[cfg(feature = "wgpu")]
unsafe impl DeviceTrait for wgpu::Device {
    #[inline]
    fn poll(&self, maintain: Maintain) {
        let maintain = match maintain {
            Maintain::Poll => wgpu::Maintain::Poll,
            Maintain::Wait => wgpu::Maintain::Wait,
        };

        self.poll(maintain);
    }

    #[inline]
    fn create_buffer(&self, desc: &crate::BufferDescriptor<Option<&'static str>>) -> crate::Buffer {
        crate::Buffer(Box::new(self.create_buffer(desc)))
    }

    #[inline]
    fn create_buffer_init(&self, desc: &crate::BufferInitDescriptor) -> crate::Buffer {
        let desc = wgpu::util::BufferInitDescriptor {
            label: desc.label,
            contents: desc.contents,
            usage: desc.usage,
        };

        crate::Buffer(Box::new(wgpu::util::DeviceExt::create_buffer_init(
            self, &desc,
        )))
    }

    #[inline]
    fn create_texture(
        &self,
        desc: &crate::TextureDescriptor<Option<&'static str>>,
    ) -> crate::Texture {
        crate::Texture(Box::new(self.create_texture(desc)))
    }

    #[inline]
    fn create_texture_with_data(
        &self,
        queue: &crate::Queue,
        desc: &crate::TextureDescriptor<Option<&'static str>>,
        data: &[u8],
    ) -> crate::Texture {
        crate::Texture(Box::new(wgpu::util::DeviceExt::create_texture_with_data(
            self,
            unsafe { &*(queue.0.as_ref() as *const _ as *const _) },
            desc,
            data,
        )))
    }

    #[inline]
    fn create_sampler(&self, desc: &crate::SamplerDescriptor) -> crate::Sampler {
        let desc = wgpu::SamplerDescriptor {
            label: desc.label,
            address_mode_u: desc.address_mode_u,
            address_mode_v: desc.address_mode_v,
            address_mode_w: desc.address_mode_w,
            mag_filter: desc.mag_filter,
            min_filter: desc.min_filter,
            mipmap_filter: desc.mipmap_filter,
            lod_min_clamp: desc.lod_min_clamp,
            lod_max_clamp: desc.lod_max_clamp,
            compare: desc.compare,
            anisotropy_clamp: desc.anisotropy_clamp,
            border_color: desc.border_color,
        };

        crate::Sampler(Box::new(self.create_sampler(&desc)))
    }

    #[inline]
    fn create_shader_module(&self, desc: &crate::ShaderModuleDescriptor) -> crate::ShaderModule {
        let desc = wgpu::ShaderModuleDescriptor {
            label: desc.label,
            source: match &desc.source {
                crate::ShaderSource::Wgsl(wgsl) => wgpu::ShaderSource::Wgsl(wgsl.clone()),
            },
        };

        crate::ShaderModule(Box::new(self.create_shader_module(&desc)))
    }

    #[inline]
    fn create_bind_group_layout(
        &self,
        desc: &crate::BindGroupLayoutDescriptor,
    ) -> crate::BindGroupLayout {
        let desc = wgpu::BindGroupLayoutDescriptor {
            label: desc.label,
            entries: desc.entries,
        };

        crate::BindGroupLayout(Box::new(self.create_bind_group_layout(&desc)))
    }

    #[inline]
    fn create_bind_group(&self, desc: &crate::BindGroupDescriptor) -> crate::BindGroup {
        let entries = desc
            .entries
            .iter()
            .map(|entry| wgpu::BindGroupEntry {
                binding: entry.binding,
                resource: match entry.resource {
                    crate::BindingResource::Buffer(ref buffer_binding) => {
                        wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                            buffer: unsafe {
                                &*(buffer_binding.buffer.0.as_ref() as *const _ as *const _)
                            },
                            offset: buffer_binding.offset,
                            size: buffer_binding.size,
                        })
                    }
                    crate::BindingResource::BufferArray(_buffers) => unimplemented!(),
                    crate::BindingResource::Sampler(sampler) => {
                        wgpu::BindingResource::Sampler(unsafe {
                            &*(sampler.0.as_ref() as *const _ as *const _)
                        })
                    }
                    crate::BindingResource::TextureView(texture_view) => {
                        wgpu::BindingResource::TextureView(unsafe {
                            &*(texture_view.0.as_ref() as *const _ as *const _)
                        })
                    }
                    crate::BindingResource::TextureViewArray(_textures) => unimplemented!(),
                },
            })
            .collect::<Vec<_>>();

        let desc = wgpu::BindGroupDescriptor {
            label: desc.label,
            layout: unsafe { &*(desc.layout.0.as_ref() as *const _ as *const _) },
            entries: &entries,
        };

        crate::BindGroup(Box::new(self.create_bind_group(&desc)))
    }

    #[inline]
    fn create_pipeline_layout(
        &self,
        desc: &crate::PipelineLayoutDescriptor,
    ) -> crate::PipelineLayout {
        let bind_group_layouts = desc
            .bind_group_layouts
            .iter()
            .map(|layout| unsafe { &*(layout.0.as_ref() as *const _ as *const _) })
            .collect::<Vec<_>>();

        let pipeline_layout = self.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: desc.label,
            bind_group_layouts: &bind_group_layouts,
            push_constant_ranges: desc.push_constant_ranges,
        });

        crate::PipelineLayout(Box::new(pipeline_layout))
    }

    #[inline]
    fn create_render_pipeline(
        &self,
        desc: &crate::RenderPipelineDescriptor,
    ) -> crate::RenderPipeline {
        let layout = desc
            .layout
            .map(|layout| unsafe { &*(layout.0.as_ref() as *const _ as *const _) });

        let vertex_module = unsafe { &*(desc.vertex.module.0.as_ref() as *const _ as *const _) };

        let buffers = desc
            .vertex
            .buffers
            .iter()
            .map(|layout| wgpu::VertexBufferLayout {
                array_stride: layout.array_stride,
                step_mode: layout.step_mode,
                attributes: layout.attributes,
            })
            .collect::<Vec<_>>();

        let fragment = desc.fragment.as_ref().map(|state| wgpu::FragmentState {
            module: unsafe { &*(state.module.0.as_ref() as *const _ as *const _) },
            entry_point: state.entry_point,
            targets: state.targets,
        });

        let desc = wgpu::RenderPipelineDescriptor {
            label: desc.label,
            layout,
            vertex: wgpu::VertexState {
                module: vertex_module,
                entry_point: desc.vertex.entry_point,
                buffers: &buffers,
            },
            fragment,
            primitive: desc.primitive,
            multisample: desc.multisample,
            depth_stencil: desc.depth_stencil.clone(),
        };

        crate::RenderPipeline(Box::new(self.create_render_pipeline(&desc)))
    }

    #[inline]
    fn create_compute_pipeline(
        &self,
        desc: &crate::ComputePipelineDescriptor,
    ) -> crate::ComputePipeline {
        let layout = desc
            .layout
            .map(|layout| unsafe { &*(layout.0.as_ref() as *const _ as *const _) });

        let module = unsafe { &*(desc.module.0.as_ref() as *const _ as *const _) };

        let desc = wgpu::ComputePipelineDescriptor {
            label: desc.label,
            layout,
            module,
            entry_point: desc.entry_point,
        };

        crate::ComputePipeline(Box::new(self.create_compute_pipeline(&desc)))
    }

    #[inline]
    fn create_command_encoder(
        &self,
        desc: &crate::CommandEncoderDescriptor<Option<&'static str>>,
    ) -> crate::CommandEncoder {
        crate::CommandEncoder(Box::new(self.create_command_encoder(desc)))
    }
}
