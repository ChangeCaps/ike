use super::D3Node;

impl D3Node {
    pub(crate) fn create_depth_pipeline(&mut self, device: &ike_wgpu::Device) {
        if self.depth_pipeline.is_some() {
            return;
        }

        let texture = device.create_texture(&ike_wgpu::TextureDescriptor {
            label: None,
            size: ike_wgpu::Extent3d {
                width: 4096,
                height: 4096,
                depth_or_array_layers: 16,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: ike_wgpu::TextureDimension::D2,
            format: ike_wgpu::TextureFormat::Depth32Float,
            usage: ike_wgpu::TextureUsages::RENDER_ATTACHMENT
                | ike_wgpu::TextureUsages::TEXTURE_BINDING,
        });

        let bind_group_layout =
            device.create_bind_group_layout(&ike_wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[
                    ike_wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        ty: ike_wgpu::BindingType::Texture {
                            sample_type: ike_wgpu::TextureSampleType::Depth,
                            view_dimension: ike_wgpu::TextureViewDimension::D2Array,
                            multisampled: false,
                        },
                        visibility: ike_wgpu::ShaderStages::VERTEX_FRAGMENT,
                        count: None,
                    },
                    ike_wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        ty: ike_wgpu::BindingType::Sampler {
                            filtering: true,
                            comparison: false,
                        },
                        visibility: ike_wgpu::ShaderStages::VERTEX_FRAGMENT,
                        count: None,
                    },
                ],
            });

        let bind_group = device.create_bind_group(&ike_wgpu::BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[
                ike_wgpu::BindGroupEntry {
                    binding: 0,
                    resource: ike_wgpu::BindingResource::TextureView(
                        &texture.create_view(&Default::default()),
                    ),
                },
                ike_wgpu::BindGroupEntry {
                    binding: 1,
                    resource: ike_wgpu::BindingResource::Sampler(&device.create_sampler(
                        &ike_wgpu::SamplerDescriptor {
                            min_filter: ike_wgpu::FilterMode::Linear,
                            mag_filter: ike_wgpu::FilterMode::Linear,
                            ..Default::default()
                        },
                    )),
                },
            ],
        });

        self.shadow_map_layout = Some(bind_group_layout);
        self.shadow_map_bind_group = Some(bind_group);

        let buffer = device.create_buffer(&ike_wgpu::BufferDescriptor {
            label: None,
            size: 64,
            mapped_at_creation: false,
            usage: ike_wgpu::BufferUsages::COPY_DST | ike_wgpu::BufferUsages::UNIFORM,
        });

        let bind_group_layout =
            device.create_bind_group_layout(&ike_wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[ike_wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    ty: ike_wgpu::BindingType::Buffer {
                        ty: ike_wgpu::BufferBindingType::Uniform,
                        min_binding_size: None,
                        has_dynamic_offset: false,
                    },
                    visibility: ike_wgpu::ShaderStages::VERTEX_FRAGMENT,
                    count: None,
                }],
            });

        let joint_matrices =
            device.create_bind_group_layout(&ike_wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[ike_wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    ty: ike_wgpu::BindingType::Buffer {
                        ty: ike_wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    visibility: ike_wgpu::ShaderStages::VERTEX_FRAGMENT,
                    count: None,
                }],
            });

        let bind_group = device.create_bind_group(&ike_wgpu::BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[ike_wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });

        let module = device.create_shader_module(&ike_wgpu::include_wgsl!("depth.wgsl"));

        let layout = device.create_pipeline_layout(&ike_wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[
                &bind_group_layout,
                self.textures_layout.as_ref().unwrap(),
                &joint_matrices,
            ],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&ike_wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&layout),
            vertex: ike_wgpu::VertexState {
                module: &module,
                entry_point: "main",
                buffers: &[
                    ike_wgpu::VertexBufferLayout {
                        array_stride: 96,
                        step_mode: ike_wgpu::VertexStepMode::Vertex,
                        attributes: &[
                            ike_wgpu::VertexAttribute {
                                format: ike_wgpu::VertexFormat::Float32x3,
                                offset: 0,
                                shader_location: 0,
                            },
                            ike_wgpu::VertexAttribute {
                                format: ike_wgpu::VertexFormat::Uint32x4,
                                offset: 64,
                                shader_location: 1,
                            },
                            ike_wgpu::VertexAttribute {
                                format: ike_wgpu::VertexFormat::Float32x4,
                                offset: 80,
                                shader_location: 2,
                            },
                        ],
                    },
                    ike_wgpu::VertexBufferLayout {
                        array_stride: 64,
                        step_mode: ike_wgpu::VertexStepMode::Instance,
                        attributes: &[
                            ike_wgpu::VertexAttribute {
                                format: ike_wgpu::VertexFormat::Float32x4,
                                offset: 0,
                                shader_location: 8,
                            },
                            ike_wgpu::VertexAttribute {
                                format: ike_wgpu::VertexFormat::Float32x4,
                                offset: 16,
                                shader_location: 9,
                            },
                            ike_wgpu::VertexAttribute {
                                format: ike_wgpu::VertexFormat::Float32x4,
                                offset: 32,
                                shader_location: 10,
                            },
                            ike_wgpu::VertexAttribute {
                                format: ike_wgpu::VertexFormat::Float32x4,
                                offset: 48,
                                shader_location: 11,
                            },
                        ],
                    },
                ],
            },
            fragment: None,
            primitive: Default::default(),
            multisample: Default::default(),
            depth_stencil: Some(ike_wgpu::DepthStencilState {
                format: ike_wgpu::TextureFormat::Depth32Float,
                depth_compare: ike_wgpu::CompareFunction::LessEqual,
                depth_write_enabled: true,
                stencil: Default::default(),
                bias: Default::default(),
            }),
        });

        self.depth_textures = Some(texture);
        self.depth_view_buffer = Some(buffer);
        self.depth_bind_group = Some(bind_group);
        self.depth_pipeline = Some(pipeline);
    }
}
