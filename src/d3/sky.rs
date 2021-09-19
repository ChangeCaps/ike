use std::collections::HashMap;

use bytemuck::bytes_of;
use glam::Mat4;

use crate::{id::{HasId, Id}, prelude::{Camera, Texture}, renderer::{Drawable, PassNode, PassNodeCtx, RenderCtx, SampleCount, TargetFormat}};

pub struct SkyTexture<'a> {
	pub texture: &'a Texture,
}

impl<'a> SkyTexture<'a> {
	#[inline]
	pub fn new(texture: &'a Texture) -> Self {
		Self { texture }
	}
}

impl Drawable for SkyTexture<'_> {
	type Node = SkyNode;

	#[inline]
	fn draw(&self, ctx: &RenderCtx, node: &mut Self::Node) {
		node.set_texture(ctx, self.texture);
	}
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    view_to_world: Mat4,
    clip_to_view: Mat4,
}

#[derive(Default)]
pub struct SkyNode {
    current_texture: Option<Id<Texture>>,
    uniform_buffer: Option<ike_wgpu::Buffer>,
    bind_group_layout: Option<ike_wgpu::BindGroupLayout>,
    bind_group: Option<ike_wgpu::BindGroup>,
    pipelines: HashMap<ike_wgpu::TextureFormat, ike_wgpu::RenderPipeline>,
}

impl SkyNode {
    #[inline]
    fn create_resources(&mut self, ctx: &RenderCtx) {
        if self.bind_group.is_none() {
            let buffer = ctx.device.create_buffer(&ike_wgpu::BufferDescriptor {
                label: None,
                size: std::mem::size_of::<Uniforms>() as u64,
                mapped_at_creation: false,
                usage: ike_wgpu::BufferUsages::COPY_DST | ike_wgpu::BufferUsages::UNIFORM,
            });

            let layout =
                ctx.device
                    .create_bind_group_layout(&ike_wgpu::BindGroupLayoutDescriptor {
                        label: None,
                        entries: &[
                            ike_wgpu::BindGroupLayoutEntry {
                                binding: 0,
                                ty: ike_wgpu::BindingType::Buffer {
                                    ty: ike_wgpu::BufferBindingType::Uniform,
                                    min_binding_size: None,
                                    has_dynamic_offset: false,
                                },
                                visibility: ike_wgpu::ShaderStages::VERTEX_FRAGMENT,
                                count: None,
                            },
                            ike_wgpu::BindGroupLayoutEntry {
                                binding: 1,
                                ty: ike_wgpu::BindingType::Texture {
                                    sample_type: ike_wgpu::TextureSampleType::Float {
                                        filterable: true,
                                    },
                                    view_dimension: ike_wgpu::TextureViewDimension::D2,
                                    multisampled: false,
                                },
                                visibility: ike_wgpu::ShaderStages::VERTEX_FRAGMENT,
                                count: None,
                            },
                            ike_wgpu::BindGroupLayoutEntry {
                                binding: 2,
                                ty: ike_wgpu::BindingType::Sampler {
                                    filtering: true,
                                    comparison: false,
                                },
                                visibility: ike_wgpu::ShaderStages::VERTEX_FRAGMENT,
                                count: None,
                            },
                        ],
                    });

            self.uniform_buffer = Some(buffer);
            self.bind_group_layout = Some(layout);
        }
    }

    #[inline]
    fn set_texture(&mut self, ctx: &RenderCtx, texture: &Texture) {
		if self.current_texture == Some(texture.id()) {
			return;
		}
		
		if self.bind_group_layout.is_none() {
			self.create_resources(ctx);
		}

        let bind_group = ctx
            .device
            .create_bind_group(&ike_wgpu::BindGroupDescriptor {
                label: None,
                layout: &self.bind_group_layout.as_ref().unwrap(),
                entries: &[
                    ike_wgpu::BindGroupEntry {
                        binding: 0,
                        resource: self.uniform_buffer.as_ref().unwrap().as_entire_binding(),
                    }, 
                    ike_wgpu::BindGroupEntry {
                        binding: 1,
                        resource: ike_wgpu::BindingResource::TextureView(
                            &texture.texture(ctx).create_view(&Default::default()),
                        ), 
                    },
                    ike_wgpu::BindGroupEntry {
                        binding: 2,
                        resource: ike_wgpu::BindingResource::Sampler(&ctx.device.create_sampler(
                            &ike_wgpu::SamplerDescriptor {
                                min_filter: ike_wgpu::FilterMode::Linear,
                                mag_filter: ike_wgpu::FilterMode::Linear,
                                ..Default::default()
                            },
                        )),
                    },
                ],
            });

        self.bind_group = Some(bind_group);
    }

    #[inline]
    fn create_pipeline(
        &self,
        device: &ike_wgpu::Device,
        format: ike_wgpu::TextureFormat,
        sample_count: u32,
    ) -> ike_wgpu::RenderPipeline {
        let module = device.create_shader_module(&ike_wgpu::include_wgsl!("sky.wgsl"));

        let layout = device.create_pipeline_layout(&ike_wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[self.bind_group_layout.as_ref().unwrap()],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&ike_wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&layout),
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
                depth_compare: ike_wgpu::CompareFunction::Always,
                depth_write_enabled: false,
                stencil: Default::default(),
                bias: Default::default(),
            }),
        });

        pipeline
    }
}

impl<S> PassNode<S> for SkyNode {
    #[inline]
    fn run<'a>(&'a mut self, ctx: &mut PassNodeCtx<'_, 'a>, _: &mut S) {
		let bind_group = if let Some(ref bind_group) = self.bind_group {
			bind_group
		} else {
			return;
		};

        let sample_count = ctx.data.get::<SampleCount>().unwrap_or(&SampleCount(1));
        let format = ctx
            .data
            .get::<TargetFormat>()
            .cloned()
            .unwrap_or_else(|| TargetFormat(ctx.view.format))
            .0;
        let camera = ctx.data.get::<Camera>().unwrap_or_else(|| &ctx.view.camera);

        let uniforms = Uniforms {
            view_to_world: camera.view,
            clip_to_view: camera.proj.inverse(),
        };

        ctx.render_ctx.queue.write_buffer(
            self.uniform_buffer.as_ref().unwrap(),
            0,
            bytes_of(&uniforms),
        );

        if !self.pipelines.contains_key(&format) {
            let pipeline = self.create_pipeline(&ctx.render_ctx.device, format, sample_count.0);

            self.pipelines.insert(format, pipeline);
        }

        let pipeline = &self.pipelines[&format];

        ctx.render_pass.set_pipeline(pipeline);

        ctx.render_pass
            .set_bind_group(0, bind_group, &[]);

        ctx.render_pass.draw(0..3, 0..1);
    }
}
