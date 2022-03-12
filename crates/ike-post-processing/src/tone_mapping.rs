use std::mem;

use bytemuck::{Pod, Zeroable};
use ike_ecs::{FromWorld, World};
use ike_render::{
    include_wgsl, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
    BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType, Buffer,
    BufferBindingType, BufferDescriptor, BufferUsages, ColorTargetState, ColorWrites,
    FragmentState, LoadOp, Operations, PipelineLayoutDescriptor, RenderContext, RenderDevice,
    RenderGraphContext, RenderGraphResult, RenderNode, RenderPassColorAttachment,
    RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor, Sampler, SamplerBindingType,
    ShaderStages, SlotInfo, TextureFormat, TextureSampleType, TextureView, TextureViewDimension,
    TextureViewId, VertexState,
};

pub struct ToneMappingPipeline {
    pub bind_group_layout: BindGroupLayout,
    pub sampler: Sampler,
    pub render_pipeline: RenderPipeline,
}

impl FromWorld for ToneMappingPipeline {
    fn from_world(world: &mut World) -> Self {
        let device = world.resource::<RenderDevice>();

        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("ike_tone_mapping_bind_group_layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: false },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    visibility: ShaderStages::VERTEX_FRAGMENT,
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    ty: BindingType::Sampler(SamplerBindingType::NonFiltering),
                    visibility: ShaderStages::VERTEX_FRAGMENT,
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    visibility: ShaderStages::VERTEX_FRAGMENT,
                    count: None,
                },
            ],
        });

        let sampler = device.create_sampler(&Default::default());

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("ike_tone_mapping_pipeline_layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let vert = device.create_shader_module(&include_wgsl!("fullscreen_quad.wgsl"));
        let frag = device.create_shader_module(&include_wgsl!("tone_mapping.wgsl"));

        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("ike_tone_mapping_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &vert,
                entry_point: "vert",
                buffers: &[],
            },
            fragment: Some(FragmentState {
                module: &frag,
                entry_point: "frag",
                targets: &[ColorTargetState {
                    format: TextureFormat::Bgra8UnormSrgb,
                    blend: None,
                    write_mask: ColorWrites::ALL,
                }],
            }),
            depth_stencil: None,
            primitive: Default::default(),
            multisample: Default::default(),
            multiview: None,
        });

        Self {
            bind_group_layout,
            sampler,
            render_pipeline,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct ToneMappingUniforms {
    pub tone_mapping: u32,
}

pub struct HdrTargetBinding {
    pub id: TextureViewId,
    pub buffer: Buffer,
    pub bind_group: BindGroup,
}

impl HdrTargetBinding {
    pub fn new(
        target: &TextureView,
        pipeline: &ToneMappingPipeline,
        device: &RenderDevice,
    ) -> Self {
        let buffer = device.create_buffer(&BufferDescriptor {
            label: Some("ike_tone_mapping_uniforms_buffer"),
            size: mem::size_of::<ToneMappingUniforms>() as u64,
            usage: BufferUsages::COPY_DST | BufferUsages::UNIFORM,
            mapped_at_creation: false,
        });

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("ike_tone_mapping_bind_group"),
            layout: &pipeline.bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(target.raw()),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&pipeline.sampler),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: buffer.raw().as_entire_binding(),
                },
            ],
        });

        Self {
            id: target.id(),
            buffer,
            bind_group,
        }
    }
}

#[derive(Default)]
pub struct ToneMappingNode {
    pub binding: Option<HdrTargetBinding>,
}

impl ToneMappingNode {
    pub const HDR_TARGET: &'static str = "hdr_target";
    pub const TARGET: &'static str = "target";
}

impl RenderNode for ToneMappingNode {
    fn input() -> Vec<SlotInfo> {
        vec![
            SlotInfo::new::<TextureView>(Self::HDR_TARGET),
            SlotInfo::new::<TextureView>(Self::TARGET),
        ]
    }

    fn run(
        &mut self,
        graph_context: &mut RenderGraphContext<'_>,
        render_context: &mut RenderContext,
        world: &World,
    ) -> RenderGraphResult<()> {
        let hdr_target = graph_context.get_input::<TextureView>(Self::HDR_TARGET)?;
        let target = graph_context.get_input::<TextureView>(Self::TARGET)?;
        let pipeline = world.resource::<ToneMappingPipeline>();

        let recreate_binding = self
            .binding
            .as_ref()
            .map_or(true, |binding| binding.id != hdr_target.id());

        if recreate_binding {
            self.binding = Some(HdrTargetBinding::new(
                hdr_target,
                &pipeline,
                &render_context.device,
            ));
        }

        let binding = self.binding.as_ref().unwrap();

        let mut render_pass = render_context
            .encoder
            .begin_render_pass(&RenderPassDescriptor {
                label: Some("ike_tone_mapping_pass"),
                color_attachments: &[RenderPassColorAttachment {
                    view: target.raw(),
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Load,
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });

        render_pass.set_pipeline(&pipeline.render_pipeline);

        render_pass.set_bind_group(0, &binding.bind_group, &[]);

        render_pass.draw(0..6, 0..1);

        Ok(())
    }
}
