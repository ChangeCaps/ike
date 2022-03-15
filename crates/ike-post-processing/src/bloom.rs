use std::{mem, num::NonZeroU32};

use bytemuck::{bytes_of, Pod, Zeroable};
use ike_ecs::{FromWorld, World};
use ike_render::{
    include_wgsl, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
    BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType, Buffer,
    BufferBindingType, BufferDescriptor, BufferUsages, ColorTargetState, ColorWrites,
    CommandEncoder, Extent3d, FragmentState, LoadOp, Operations, PipelineLayout,
    PipelineLayoutDescriptor, RenderContext, RenderDevice, RenderGraphContext, RenderGraphResult,
    RenderNode, RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline,
    RenderPipelineDescriptor, Sampler, SamplerBindingType, SamplerDescriptor, ShaderModule,
    ShaderStages, SlotInfo, Texture, TextureDescriptor, TextureDimension, TextureFormat, TextureId,
    TextureSampleType, TextureUsages, TextureView, TextureViewDescriptor, TextureViewDimension,
    VertexState,
};

fn pipeline(
    device: &RenderDevice,
    module: &ShaderModule,
    entry_point: &str,
    layout: &PipelineLayout,
) -> RenderPipeline {
    let vert = device.create_shader_module(&include_wgsl!("fullscreen_quad.wgsl"));

    device.create_render_pipeline(&RenderPipelineDescriptor {
        label: Some("ike_bloom"),
        layout: Some(layout),
        vertex: VertexState {
            module: &vert,
            entry_point: "vert",
            buffers: &[],
        },
        fragment: Some(FragmentState {
            module,
            entry_point,
            targets: &[ColorTargetState {
                format: TextureFormat::Rgba32Float,
                blend: None,
                write_mask: ColorWrites::all(),
            }],
        }),
        depth_stencil: None,
        primitive: Default::default(),
        multisample: Default::default(),
        multiview: None,
    })
}

pub struct BloomPipeline {
    pub down_sample_layout: BindGroupLayout,
    pub up_sample_layout: BindGroupLayout,
    pub sampler: Sampler,
    pub filter_pipeline: RenderPipeline,
    pub down_sample_pipeline: RenderPipeline,
    pub up_sample_pipeline: RenderPipeline,
    pub apply_pipeline: RenderPipeline,
}

impl FromWorld for BloomPipeline {
    fn from_world(world: &mut World) -> Self {
        let device = world.resource::<RenderDevice>();

        let down_sample_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("ike_bloom_down_sample_bind_group_layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: false },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    visibility: ShaderStages::FRAGMENT,
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    ty: BindingType::Sampler(SamplerBindingType::NonFiltering),
                    visibility: ShaderStages::FRAGMENT,
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 3,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    visibility: ShaderStages::FRAGMENT,
                    count: None,
                },
            ],
        });

        let up_sample_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("ike_bloom_up_sample_bind_group_layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: false },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    visibility: ShaderStages::FRAGMENT,
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: false },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    visibility: ShaderStages::FRAGMENT,
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    ty: BindingType::Sampler(SamplerBindingType::NonFiltering),
                    visibility: ShaderStages::FRAGMENT,
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 3,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    visibility: ShaderStages::FRAGMENT,
                    count: None,
                },
            ],
        });

        let sampler = device.create_sampler(&SamplerDescriptor {
            ..Default::default()
        });

        let module = &device.create_shader_module(&include_wgsl!("bloom.wgsl"));

        let down_sample_pipeline_layout =
            device.create_pipeline_layout(&PipelineLayoutDescriptor {
                label: Some("ike_bloom_down_sample_pipeline_layout"),
                bind_group_layouts: &[&down_sample_layout],
                push_constant_ranges: &[],
            });

        let up_sample_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("ike_bloom_up_sample_pipeline_layout"),
            bind_group_layouts: &[&up_sample_layout],
            push_constant_ranges: &[],
        });

        let filter_pipeline = pipeline(&device, module, "filter", &down_sample_pipeline_layout);
        let down_sample_pipeline =
            pipeline(&device, module, "down_sample", &down_sample_pipeline_layout);
        let up_sample_pipeline = pipeline(&device, module, "up_sample", &up_sample_pipeline_layout);
        let apply_pipeline = pipeline(&device, module, "apply", &up_sample_pipeline_layout);

        Self {
            down_sample_layout,
            up_sample_layout,
            sampler,
            filter_pipeline,
            down_sample_pipeline,
            up_sample_pipeline,
            apply_pipeline,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct BloomUniforms {
    pub threshold: f32,
    pub knee: f32,
    pub scale: f32,
}

#[derive(Default)]
pub struct BloomNode {
    pub uniforms: Option<Buffer>,
    pub target_id: Option<TextureId>,
    pub texture_size: Option<[u32; 2]>,
    pub output_texture: Option<Texture>,
    pub down_sample_texture: Option<Texture>,
    pub up_sample_texture: Option<Texture>,
    pub filter_bind_group: Option<BindGroup>,
    pub down_sample_bind_groups: Vec<BindGroup>,
    pub up_sample_bind_groups: Vec<BindGroup>,
    pub apply_bind_group: Option<BindGroup>,
}

impl BloomNode {
    pub const TARGET: &'static str = "target";
    pub const OUTPUT: &'static str = "output";

    pub fn calculate_mips(width: u32, height: u32) -> u32 {
        let size = u32::min(width, height);
        let mut mips = 1;

        while size / 2u32.pow(mips) > 4 {
            mips += 1;
        }

        mips + 1
    }

    pub fn scale_factor(width: u32, height: u32, mips: u32) -> f32 {
        let size = u32::min(width, height);

        (size / 2u32.pow(mips - 1)) as f32 / 4.0
    }

    pub fn recreate_resources(
        &mut self,
        device: &RenderDevice,
        pipeline: &BloomPipeline,
        target: &TextureView,
        width: u32,
        height: u32,
    ) {
        if self.uniforms.is_none() {
            let buffer = device.create_buffer(&BufferDescriptor {
                label: Some("ike_bloom_uniforms_buffer"),
                size: mem::size_of::<BloomUniforms>() as u64,
                usage: BufferUsages::COPY_DST | BufferUsages::UNIFORM,
                mapped_at_creation: false,
            });

            self.uniforms = Some(buffer);
        }

        let uniforms = self.uniforms.as_ref().unwrap();

        let mips = Self::calculate_mips(width, height);

        let output_texture = device.create_texture(&TextureDescriptor {
            label: Some("ike_bloom_output_texture"),
            size: Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba32Float,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::RENDER_ATTACHMENT,
        });

        let down_sample_texture = device.create_texture(&TextureDescriptor {
            label: Some("ike_bloom_down_sample_texture"),
            size: Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: mips,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba32Float,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::RENDER_ATTACHMENT,
        });

        let up_sample_texture = device.create_texture(&TextureDescriptor {
            label: Some("ike_bloom_up_sample_texture"),
            size: Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: mips,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba32Float,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::RENDER_ATTACHMENT,
        });

        let filter_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("ike_bloom_filter_bind_group"),
            layout: &pipeline.down_sample_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(target.raw()),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::Sampler(&pipeline.sampler),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: uniforms.raw().as_entire_binding(),
                },
            ],
        });

        self.filter_bind_group = Some(filter_bind_group);

        self.down_sample_bind_groups.clear();

        for mip in 0..mips - 1 {
            let source = down_sample_texture.create_view(&TextureViewDescriptor {
                base_mip_level: mip,
                mip_level_count: NonZeroU32::new(1),
                ..Default::default()
            });

            let bind_group = device.create_bind_group(&BindGroupDescriptor {
                label: Some("ike_bloom_down_sample_bind_group"),
                layout: &pipeline.down_sample_layout,
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: BindingResource::TextureView(source.raw()),
                    },
                    BindGroupEntry {
                        binding: 2,
                        resource: BindingResource::Sampler(&pipeline.sampler),
                    },
                    BindGroupEntry {
                        binding: 3,
                        resource: uniforms.raw().as_entire_binding(),
                    },
                ],
            });

            self.down_sample_bind_groups.push(bind_group);
        }

        self.up_sample_bind_groups.clear();

        for mip in 0..mips - 1 {
            let source_texture = if mip == mips - 2 {
                &down_sample_texture
            } else {
                &up_sample_texture
            };

            let source = source_texture.create_view(&TextureViewDescriptor {
                base_mip_level: mip + 1,
                mip_level_count: NonZeroU32::new(1),
                ..Default::default()
            });

            let additional = down_sample_texture.create_view(&TextureViewDescriptor {
                base_mip_level: mip,
                mip_level_count: NonZeroU32::new(1),
                ..Default::default()
            });

            let bind_group = device.create_bind_group(&BindGroupDescriptor {
                label: Some("ike_up_down_sample_bind_group"),
                layout: &pipeline.up_sample_layout,
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: BindingResource::TextureView(source.raw()),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: BindingResource::TextureView(additional.raw()),
                    },
                    BindGroupEntry {
                        binding: 2,
                        resource: BindingResource::Sampler(&pipeline.sampler),
                    },
                    BindGroupEntry {
                        binding: 3,
                        resource: uniforms.raw().as_entire_binding(),
                    },
                ],
            });

            self.up_sample_bind_groups.push(bind_group);
        }

        let additional = up_sample_texture.create_view(&TextureViewDescriptor {
            base_array_layer: 0,
            array_layer_count: NonZeroU32::new(1),
            ..Default::default()
        });

        let apply_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("ike_bloom_apply_bind_group"),
            layout: &pipeline.up_sample_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(target.raw()),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(additional.raw()),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::Sampler(&pipeline.sampler),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: uniforms.raw().as_entire_binding(),
                },
            ],
        });

        self.apply_bind_group = Some(apply_bind_group);

        self.output_texture = Some(output_texture);
        self.down_sample_texture = Some(down_sample_texture);
        self.up_sample_texture = Some(up_sample_texture);
    }
}

fn render_pass(
    encoder: &mut CommandEncoder,
    pipeline: &RenderPipeline,
    bind_group: &BindGroup,
    target: &TextureView,
) {
    let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
        label: Some("ike_bloom_pass"),
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

    render_pass.set_pipeline(pipeline);
    render_pass.set_bind_group(0, bind_group, &[]);
    render_pass.draw(0..6, 0..1);
}

impl RenderNode for BloomNode {
    fn input() -> Vec<SlotInfo> {
        vec![SlotInfo::new::<TextureView>(Self::TARGET)]
    }

    fn output() -> Vec<SlotInfo> {
        vec![SlotInfo::new::<TextureView>(Self::OUTPUT)]
    }

    fn run(
        &mut self,
        graph_context: &mut RenderGraphContext<'_>,
        render_context: &mut RenderContext,
        world: &World,
    ) -> RenderGraphResult<()> {
        let target = graph_context.get_input::<TextureView>(Self::TARGET)?;
        let pipeline = world.resource::<BloomPipeline>();

        if self.texture_size != Some([target.width(), target.height()]) {
            self.texture_size = Some([target.width(), target.height()]);

            self.recreate_resources(
                &render_context.device,
                &pipeline,
                target,
                target.width(),
                target.height(),
            );
        }

        let mips = Self::calculate_mips(target.width(), target.height());

        let uniforms = self.uniforms.as_ref().unwrap();

        render_context.queue.write_buffer(
            &uniforms,
            0,
            bytes_of(&BloomUniforms {
                threshold: 2.5,
                knee: 0.5,
                scale: Self::scale_factor(target.width(), target.height(), mips),
            }),
        );

        let filter_target =
            self.down_sample_texture
                .as_ref()
                .unwrap()
                .create_view(&TextureViewDescriptor {
                    base_mip_level: 0,
                    mip_level_count: NonZeroU32::new(1),
                    ..Default::default()
                });

        render_pass(
            &mut render_context.encoder,
            &pipeline.filter_pipeline,
            self.filter_bind_group.as_ref().unwrap(),
            &filter_target,
        );

        for mip in 0..mips - 1 {
            let target =
                self.down_sample_texture
                    .as_ref()
                    .unwrap()
                    .create_view(&TextureViewDescriptor {
                        base_mip_level: mip + 1,
                        mip_level_count: NonZeroU32::new(1),
                        ..Default::default()
                    });

            render_pass(
                &mut render_context.encoder,
                &pipeline.down_sample_pipeline,
                &self.down_sample_bind_groups[mip as usize],
                &target,
            );
        }

        for mip in (0..mips - 1).rev() {
            let target =
                self.up_sample_texture
                    .as_ref()
                    .unwrap()
                    .create_view(&TextureViewDescriptor {
                        base_mip_level: mip,
                        mip_level_count: NonZeroU32::new(1),
                        ..Default::default()
                    });

            render_pass(
                &mut render_context.encoder,
                &pipeline.up_sample_pipeline,
                &self.up_sample_bind_groups[mip as usize],
                &target,
            );
        }

        let output = self
            .output_texture
            .as_ref()
            .unwrap()
            .create_view(&Default::default());

        render_pass(
            &mut render_context.encoder,
            &pipeline.apply_pipeline,
            self.apply_bind_group.as_ref().unwrap(),
            &output,
        );

        graph_context.set_output(output, Self::OUTPUT)?;

        Ok(())
    }
}
