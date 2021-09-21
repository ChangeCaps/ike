use std::collections::HashMap;

use glam::UVec2;
use ike_wgpu::CommandEncoder;

use crate::{
    prelude::View,
    renderer::{
        PassData, PassNode, PassNodeCtx, RenderCtx, RenderPass, SampleCount, TargetFormat,
        TargetSize,
    },
};

pub struct HdrTarget {
    pub view: ike_wgpu::TextureView,
    pub texture: ike_wgpu::Texture,
    pub depth: ike_wgpu::TextureView,
    pub recreated: bool,
}

#[derive(Default)]
pub struct HdrPass {
    pub size: UVec2,
    pub sample_count: u32,
    pub target: Option<ike_wgpu::TextureView>,
    pub depth: Option<ike_wgpu::TextureView>,
}

impl<S> RenderPass<S> for HdrPass {
    #[inline]
    fn run<'a>(
        &'a mut self,
        encoder: &'a mut CommandEncoder,
        ctx: &RenderCtx,
        view: &'a View,
        data: &mut PassData,
        _state: &mut S,
    ) -> ike_wgpu::RenderPass<'a> {
        let target_size = UVec2::new(view.width, view.height);

        if self.size != target_size {
            self.size = target_size;

            let texture = ctx.device.create_texture(&ike_wgpu::TextureDescriptor {
                label: None,
                size: ike_wgpu::Extent3d {
                    width: target_size.x,
                    height: target_size.y,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: ike_wgpu::TextureDimension::D2,
                format: ike_wgpu::TextureFormat::Rgba32Float,
                usage: ike_wgpu::TextureUsages::RENDER_ATTACHMENT
                    | ike_wgpu::TextureUsages::STORAGE_BINDING
                    | ike_wgpu::TextureUsages::TEXTURE_BINDING
                    | ike_wgpu::TextureUsages::COPY_SRC
                    | ike_wgpu::TextureUsages::COPY_DST,
            });

            let depth = ctx.device.create_texture(&ike_wgpu::TextureDescriptor {
                label: None,
                size: ike_wgpu::Extent3d {
                    width: target_size.x,
                    height: target_size.y,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: ike_wgpu::TextureDimension::D2,
                format: ike_wgpu::TextureFormat::Depth24Plus,
                usage: ike_wgpu::TextureUsages::RENDER_ATTACHMENT
                    | ike_wgpu::TextureUsages::TEXTURE_BINDING,
            });

            self.target = Some(texture.create_view(&Default::default()));
            self.depth = Some(depth.create_view(&Default::default()));

            data.insert(HdrTarget {
                view: texture.create_view(&Default::default()),
                texture,
                depth: depth.create_view(&Default::default()),
                recreated: true,
            });
        } else {
            if let Some(target) = data.get_mut::<HdrTarget>() {
                target.recreated = false;
            }
        }

        data.insert(SampleCount(1));
        data.insert(TargetFormat(ike_wgpu::TextureFormat::Rgba32Float));
        data.insert(TargetSize(self.size));
        data.insert(view.camera.clone());

        let target = self.target.as_ref().unwrap();
        let depth = self.depth.as_ref().unwrap();

        encoder.begin_render_pass(&ike_wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[ike_wgpu::RenderPassColorAttachment {
                view: target,
                resolve_target: None,
                ops: ike_wgpu::Operations {
                    load: ike_wgpu::LoadOp::Clear(ike_wgpu::Color::TRANSPARENT),
                    store: true,
                },
            }],
            depth_stencil_attachment: Some(ike_wgpu::RenderPassDepthStencilAttachment {
                view: depth,
                depth_ops: Some(ike_wgpu::Operations {
                    load: ike_wgpu::LoadOp::Clear(1.0),
                    store: true,
                }),
                stencil_ops: None,
            }),
        })
    }
}

#[derive(Default)]
pub struct HdrCombineNode {
    pub bind_group: Option<ike_wgpu::BindGroup>,
    pub bind_group_layout: Option<ike_wgpu::BindGroupLayout>,
    pub pipelines: HashMap<ike_wgpu::TextureFormat, ike_wgpu::RenderPipeline>,
}

impl HdrCombineNode {
    #[inline]
    pub fn new() -> Self {
        Default::default()
    }

    #[inline]
    pub fn create_pipeline(
        &mut self,
        ctx: &RenderCtx,
        format: ike_wgpu::TextureFormat,
        sample_count: u32,
    ) {
        if !self.pipelines.contains_key(&format) {
            let layout =
                ctx.device
                    .create_bind_group_layout(&ike_wgpu::BindGroupLayoutDescriptor {
                        label: None,
                        entries: &[
                            ike_wgpu::BindGroupLayoutEntry {
                                binding: 0,
                                ty: ike_wgpu::BindingType::Texture {
                                    sample_type: ike_wgpu::TextureSampleType::Float {
                                        filterable: false,
                                    },
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
                                    multisampled: false,
                                },
                                visibility: ike_wgpu::ShaderStages::VERTEX_FRAGMENT,
                                count: None,
                            },
                        ],
                    });

            let pipeline_layout =
                ctx.device
                    .create_pipeline_layout(&ike_wgpu::PipelineLayoutDescriptor {
                        label: None,
                        bind_group_layouts: &[&layout],
                        push_constant_ranges: &[],
                    });

            let module = ctx
                .device
                .create_shader_module(&ike_wgpu::include_wgsl!("hdr_combine.wgsl"));

            let render_pipeline =
                ctx.device
                    .create_render_pipeline(&ike_wgpu::RenderPipelineDescriptor {
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

impl<S> PassNode<S> for HdrCombineNode {
    #[inline]
    fn run<'a>(&'a mut self, ctx: &mut PassNodeCtx<'_, 'a>, _state: &mut S) {
        let format = ctx
            .data
            .get::<TargetFormat>()
            .map_or_else(|| ctx.view.format, |f| f.0);
        let sample_count = ctx.data.get::<SampleCount>().map_or(0, |f| f.0);

        self.create_pipeline(ctx.render_ctx, format, sample_count);

        if let Some(target) = ctx.data.get::<HdrTarget>() {
            if target.recreated {
                let bind_group =
                    ctx.render_ctx
                        .device
                        .create_bind_group(&ike_wgpu::BindGroupDescriptor {
                            label: None,
                            layout: self.bind_group_layout.as_ref().unwrap(),
                            entries: &[
                                ike_wgpu::BindGroupEntry {
                                    binding: 0,
                                    resource: ike_wgpu::BindingResource::TextureView(&target.view),
                                },
                                ike_wgpu::BindGroupEntry {
                                    binding: 1,
                                    resource: ike_wgpu::BindingResource::TextureView(&target.depth),
                                },
                            ],
                        });

                self.bind_group = Some(bind_group);
            }
        }

        let pipeline = self.pipelines.get(&format).unwrap();

        ctx.render_pass.set_pipeline(pipeline);

        ctx.render_pass
            .set_bind_group(0, self.bind_group.as_ref().unwrap(), &[]);

        ctx.render_pass.draw(0..3, 0..1);
    }
}
