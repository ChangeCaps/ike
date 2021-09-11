#[inline]
pub fn default_pipeline<S>(
    _s: &mut S,
    device: &ike_wgpu::Device,
    format: ike_wgpu::TextureFormat,
    sample_count: u32,
) -> ike_wgpu::RenderPipeline {
    let module = device.create_shader_module(&ike_wgpu::include_wgsl!("default_shader.wgsl"));

    let uniforms = device.create_bind_group_layout(&ike_wgpu::BindGroupLayoutDescriptor {
        label: None,
        entries: &[ike_wgpu::BindGroupLayoutEntry {
            binding: 0,
            ty: ike_wgpu::BindingType::Buffer {
                ty: ike_wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false, 
                min_binding_size: None,
            },
            visibility: ike_wgpu::ShaderStages::VERTEX_FRAGMENT,
            count: None,
        }],
    });

    let layout = device.create_pipeline_layout(&ike_wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[&uniforms],
        push_constant_ranges: &[],
    });

    let pipeline = device.create_render_pipeline(&ike_wgpu::RenderPipelineDescriptor {
        label: None,
        layout: Some(&layout),
        vertex: ike_wgpu::VertexState {
            module: &module,
            buffers: &[
                ike_wgpu::VertexBufferLayout {
                    array_stride: 48,
                    step_mode: ike_wgpu::VertexStepMode::Vertex,
                    attributes: &[
                        ike_wgpu::VertexAttribute {
                            format: ike_wgpu::VertexFormat::Float32x3,
                            offset: 0,
                            shader_location: 0,
                        },
                        ike_wgpu::VertexAttribute {
                            format: ike_wgpu::VertexFormat::Float32x3,
                            offset: 12,
                            shader_location: 1,
                        },
                        ike_wgpu::VertexAttribute {
                            format: ike_wgpu::VertexFormat::Float32x2,
                            offset: 24,
                            shader_location: 2,
                        },
                        ike_wgpu::VertexAttribute {
                            format: ike_wgpu::VertexFormat::Float32x4,
                            offset: 32,
                            shader_location: 3,
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
                            shader_location: 4,
                        },
                        ike_wgpu::VertexAttribute {
                            format: ike_wgpu::VertexFormat::Float32x4,
                            offset: 16,
                            shader_location: 5,
                        },
                        ike_wgpu::VertexAttribute {
                            format: ike_wgpu::VertexFormat::Float32x4,
                            offset: 32,
                            shader_location: 6,
                        },
                        ike_wgpu::VertexAttribute {
                            format: ike_wgpu::VertexFormat::Float32x4,
                            offset: 48,
                            shader_location: 7,
                        },
                    ],
                },
            ],
            entry_point: "main",
        },
        fragment: Some(ike_wgpu::FragmentState {
            module: &module,
            targets: &[ike_wgpu::ColorTargetState {
                format,
                blend: Some(ike_wgpu::BlendState::ALPHA_BLENDING),
                write_mask: ike_wgpu::ColorWrites::ALL,
            }],
            entry_point: "main",
        }),
        primitive: Default::default(),
        multisample: ike_wgpu::MultisampleState {
            count: sample_count,
            ..Default::default()
        },
        depth_stencil: Some(ike_wgpu::DepthStencilState {
            format: ike_wgpu::TextureFormat::Depth24Plus,
            depth_write_enabled: true,
            depth_compare: ike_wgpu::CompareFunction::LessEqual,
            stencil: Default::default(),
            bias: Default::default(),
        }),
    });

    pipeline
}
