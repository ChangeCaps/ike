use std::collections::HashMap;

use crate::{
    avg_luminance_node::LuminanceBuffer,
    render_texture::RenderTexture,
    renderer::{render_device, GraphError, NodeEdge, RenderNode},
};

#[derive(Default)]
pub struct HdrCombineNode {
    pub bind_group: Option<ike_wgpu::BindGroup>,
    pub bind_group_layout: Option<ike_wgpu::BindGroupLayout>,
    pub pipelines: HashMap<ike_wgpu::TextureFormat, ike_wgpu::RenderPipeline>,
}

impl HdrCombineNode {
    pub const HDR_TEXTURE: &'static str = "hdr_texture";
    pub const DEPTH_TEXTURE: &'static str = "depth_texture";
    pub const TARGET: &'static str = "target";
    pub const LUMINANCE_BUFFER: &'static str = "luminance_buffer";

    #[inline]
    pub fn new() -> Self {
        Default::default()
    }

    #[inline]
    pub fn create_pipeline(&mut self, format: ike_wgpu::TextureFormat, sample_count: u32) {
        if !self.pipelines.contains_key(&format) {
            let device = render_device();

            let layout = device.create_bind_group_layout(&ike_wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[
                    ike_wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        ty: ike_wgpu::BindingType::Texture {
                            sample_type: ike_wgpu::TextureSampleType::Float { filterable: false },
                            view_dimension: ike_wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        visibility: ike_wgpu::ShaderStages::VERTEX_FRAGMENT,
                        count: None,
                    },
                    ike_wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        ty: ike_wgpu::BindingType::Texture {
                            sample_type: ike_wgpu::TextureSampleType::Depth,
                            view_dimension: ike_wgpu::TextureViewDimension::D2,
                            multisampled: true,
                        },
                        visibility: ike_wgpu::ShaderStages::VERTEX_FRAGMENT,
                        count: None,
                    },
                    ike_wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        ty: ike_wgpu::BindingType::Buffer {
                            ty: ike_wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        visibility: ike_wgpu::ShaderStages::VERTEX_FRAGMENT,
                        count: None,
                    },
                ],
            });

            let pipeline_layout =
                device.create_pipeline_layout(&ike_wgpu::PipelineLayoutDescriptor {
                    label: None,
                    bind_group_layouts: &[&layout],
                    push_constant_ranges: &[],
                });

            let module = device.create_shader_module(&ike_wgpu::include_wgsl!("hdr_combine.wgsl"));

            let render_pipeline =
                device.create_render_pipeline(&ike_wgpu::RenderPipelineDescriptor {
                    label: None,
                    layout: Some(&pipeline_layout),
                    vertex: ike_wgpu::VertexState {
                        module: &module,
                        entry_point: "main",
                        buffers: &[],
                    },
                    fragment: Some(ike_wgpu::FragmentState {
                        module: &module,
                        entry_point: "main",
                        targets: &[ike_wgpu::ColorTargetState {
                            format,
                            blend: None,
                            write_mask: ike_wgpu::ColorWrites::ALL,
                        }],
                    }),
                    primitive: Default::default(),
                    multisample: ike_wgpu::MultisampleState {
                        count: sample_count,
                        ..Default::default()
                    },
                    depth_stencil: Some(ike_wgpu::DepthStencilState {
                        format: ike_wgpu::TextureFormat::Depth24Plus,
                        depth_compare: ike_wgpu::CompareFunction::LessEqual,
                        depth_write_enabled: true,
                        stencil: Default::default(),
                        bias: Default::default(),
                    }),
                });

            self.bind_group_layout = Some(layout);

            self.pipelines.insert(format, render_pipeline);
        }
    }
}

impl RenderNode for HdrCombineNode {
    #[inline]
    fn run(
        &mut self,
        encoder: &mut ike_wgpu::CommandEncoder,
        input: &NodeEdge,
        output: &mut NodeEdge,
    ) -> Result<(), GraphError> {
        let hdr_texture = input.get::<RenderTexture>(Self::HDR_TEXTURE)?;
        let depth_texture = input.get::<RenderTexture>(Self::DEPTH_TEXTURE)?;
        let target = input.get::<RenderTexture>(Self::TARGET)?;
        let lum = input.get::<LuminanceBuffer>(Self::LUMINANCE_BUFFER)?;

        let hdr_view = hdr_texture.texture().create_view(&Default::default());
        let depth_view = depth_texture.texture().create_view(&Default::default());

        let view = target.texture().create_view(&Default::default());

        self.create_pipeline(target.format, target.samples);

        if target.recreate {
            let bind_group = render_device().create_bind_group(&ike_wgpu::BindGroupDescriptor {
                label: None,
                layout: self.bind_group_layout.as_ref().unwrap(),
                entries: &[
                    ike_wgpu::BindGroupEntry {
                        binding: 0,
                        resource: ike_wgpu::BindingResource::TextureView(&hdr_view),
                    },
                    ike_wgpu::BindGroupEntry {
                        binding: 1,
                        resource: ike_wgpu::BindingResource::TextureView(&depth_view),
                    },
                    ike_wgpu::BindGroupEntry {
                        binding: 2,
                        resource: lum.buffer.as_entire_binding(),
                    },
                ],
            });

            self.bind_group = Some(bind_group);
        }

        let pipeline = self.pipelines.get(&target.format).unwrap();

        let mut render_pass = encoder.begin_render_pass(&ike_wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[ike_wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: ike_wgpu::Operations {
                    load: ike_wgpu::LoadOp::Load,
                    store: true,
                },
            }],
            depth_stencil_attachment: None,
        });

        render_pass.set_pipeline(pipeline);

        render_pass.set_bind_group(0, self.bind_group.as_ref().unwrap(), &[]);

        render_pass.draw(0..3, 0..1);

        Ok(())
    }
}
