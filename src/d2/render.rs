use super::{
    sprite::{Sprite, Sprites},
    transform2d::Transform2d,
};
use crate::{
    color::Color,
    id::Id,
    renderer::{RenderCtx, RenderNode},
    texture::Texture,
    view::View,
};
use bytemuck::{cast_slice, Pod, Zeroable};
use glam::Vec2;
use std::collections::HashMap;
use wgpu::util::DeviceExt;

fn crate_pipeline(
    ctx: &RenderCtx,
    format: wgpu::TextureFormat,
    sample_count: u32,
) -> wgpu::RenderPipeline {
    let shader_module = ctx
        .device
        .create_shader_module(&wgpu::include_wgsl!("shader.wgsl"));

    let bind_group_layout = ctx
        .device
        .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("2d_bind_group_layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Sampler {
                        filtering: true,
                        comparison: false,
                    },
                    count: None,
                },
            ],
        });

    let layout = ctx
        .device
        .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("2d_pipeline_layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

    let pipeline = ctx
        .device
        .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("2d_pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader_module,
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: 36,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x3,
                            offset: 0,
                            shader_location: 0,
                        },
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x2,
                            offset: 12,
                            shader_location: 1,
                        },
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x4,
                            offset: 20,
                            shader_location: 2,
                        },
                    ],
                }],
                entry_point: "main",
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader_module,
                targets: &[wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                }],
                entry_point: "main",
            }),
            primitive: Default::default(),
            multisample: wgpu::MultisampleState {
                count: sample_count,
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth24Plus,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::LessEqual,
                stencil: Default::default(),
                bias: Default::default(),
            }),
        });

    pipeline
}

pub struct Render2dCtx<'a> {
    pub sprites: &'a mut Sprites,
    pub render_ctx: &'a RenderCtx,
}

impl<'a> Render2dCtx<'a> {
    #[inline]
    pub fn draw_sprite(&mut self, sprite: Sprite) {
        self.sprites.draw(sprite);
    }

    #[inline]
    pub fn draw_texture(&mut self, texture: &mut Texture, transform: &Transform2d) {
        let view = texture
            .texture(self.render_ctx)
            .create_view(&Default::default());

        let sprite = Sprite {
            transform: transform.matrix(),
            width: texture.width,
            height: texture.height,
            depth: 0.0,
            min: Vec2::ZERO,
            max: Vec2::ONE,
            texture_id: texture.id,
            view,
        };

        self.sprites.draw(sprite)
    }

    #[inline]
    pub fn draw_texture_offset(
        &mut self,
        texture: &mut Texture,
        transform: &Transform2d,
        offset: Vec2,
    ) {
        let mut transform = transform.clone();
        transform.translation += offset;

        self.draw_texture(texture, &transform);
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Vertex2d {
    position: [f32; 3],
    uv: [f32; 2],
    color: [f32; 4],
}

pub trait Render2d {
    fn render(&mut self, ctx: &mut Render2dCtx);
}

pub struct Node2d {
    clear_color: Color,
    sample_count: u32,
    width: u32,
    height: u32,
    depth_texture: Option<wgpu::TextureView>,
    ms_texture: Option<wgpu::TextureView>,
    bind_groups: HashMap<Id, wgpu::BindGroup>,
    pipelines: HashMap<wgpu::TextureFormat, wgpu::RenderPipeline>,
}

impl Default for Node2d {
    #[inline]
    fn default() -> Self {
        Self {
            clear_color: Color::BLACK,
            sample_count: 1,
            width: 0,
            height: 0,
            depth_texture: None,
            ms_texture: None,
            bind_groups: Default::default(),
            pipelines: Default::default(),
        }
    }
}

impl Node2d {
    #[inline]
    pub fn new(clear_color: Color, sample_count: u32) -> Self {
        Self {
            clear_color,
            sample_count,
            ..Default::default()
        }
    }
}

impl<S: Render2d> RenderNode<S> for Node2d {
    #[inline]
    fn run(&mut self, ctx: &RenderCtx, view: &View, state: &mut S) {
        let depth = if let Some(ref mut depth) = self.depth_texture {
            if self.width != view.width || self.height != view.height {
                let texture = ctx.device.create_texture(&wgpu::TextureDescriptor {
                    label: Some("2d_pass_depth"),
                    size: wgpu::Extent3d {
                        width: view.width,
                        height: view.height,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: self.sample_count,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::Depth24Plus,
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                });

                let texture_view = texture.create_view(&Default::default());

                if self.sample_count > 1 {
                    let texture = ctx.device.create_texture(&wgpu::TextureDescriptor {
                        label: Some("2d_pass_ms"),
                        size: wgpu::Extent3d {
                            width: view.width,
                            height: view.height,
                            depth_or_array_layers: 1,
                        },
                        mip_level_count: 1,
                        sample_count: self.sample_count,
                        dimension: wgpu::TextureDimension::D2,
                        format: view.format,
                        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                    });

                    let texture_view = texture.create_view(&Default::default());

                    self.ms_texture = Some(texture_view);
                }

                self.width = view.width;
                self.height = view.height;

                *depth = texture_view;
            }

            depth
        } else {
            let texture = ctx.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("2d_pass_depth"),
                size: wgpu::Extent3d {
                    width: view.width,
                    height: view.height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: self.sample_count,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Depth24Plus,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            });

            let texture_view = texture.create_view(&Default::default());

            self.depth_texture = Some(texture_view);

            if self.sample_count > 1 {
                let texture = ctx.device.create_texture(&wgpu::TextureDescriptor {
                    label: Some("2d_pass_ms"),
                    size: wgpu::Extent3d {
                        width: view.width,
                        height: view.height,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: self.sample_count,
                    dimension: wgpu::TextureDimension::D2,
                    format: view.format,
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                });

                let texture_view = texture.create_view(&Default::default());

                self.ms_texture = Some(texture_view);
            }

            self.width = view.width;
            self.height = view.height;

            self.depth_texture.as_mut().unwrap()
        };

        let sample_count = self.sample_count;
        let pipeline = self
            .pipelines
            .entry(view.format)
            .or_insert_with(|| crate_pipeline(ctx, view.format, sample_count));

        let mut sprites = Sprites::default();

        let mut render_ctx = Render2dCtx {
            sprites: &mut sprites,
            render_ctx: ctx,
        };

        state.render(&mut render_ctx);

        let mut encoder = ctx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("2d_pass_encoder"),
            });

        let mut buffers = Vec::new();

        let color_attachment = if self.sample_count > 1 {
            wgpu::RenderPassColorAttachment {
                view: self.ms_texture.as_ref().unwrap(),
                resolve_target: Some(&view.target),
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(self.clear_color.into()),
                    store: true,
                },
            }
        } else {
            wgpu::RenderPassColorAttachment {
                view: &view.target,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(self.clear_color.into()),
                    store: true,
                },
            }
        };

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("2d_pass"),
            color_attachments: &[color_attachment],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: depth,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: true,
                }),
                stencil_ops: None,
            }),
        });

        render_pass.set_pipeline(pipeline);

        for (id, sprites) in &sprites.batches {
            let mut vertices = Vec::with_capacity(sprites.len());

            for sprite in sprites {
                let w = sprite.width as f32 / 2.0;
                let h = sprite.height as f32 / 2.0;

                let bl = Vec2::new(-w, -h);
                let tl = Vec2::new(-w, h);
                let br = Vec2::new(w, -h);
                let tr = Vec2::new(w, h);

                let bl = sprite.transform.transform_point2(bl);
                let tl = sprite.transform.transform_point2(tl);
                let br = sprite.transform.transform_point2(br);
                let tr = sprite.transform.transform_point2(tr);

                let bl = view.view_proj.transform_point3(bl.extend(sprite.depth));
                let tl = view.view_proj.transform_point3(tl.extend(sprite.depth));
                let br = view.view_proj.transform_point3(br.extend(sprite.depth));
                let tr = view.view_proj.transform_point3(tr.extend(sprite.depth));

                vertices.push(Vertex2d {
                    position: bl.into(),
                    uv: [sprite.min.x, sprite.max.y],
                    color: [1.0; 4],
                });
                vertices.push(Vertex2d {
                    position: tl.into(),
                    uv: [sprite.min.x, sprite.min.y],
                    color: [1.0; 4],
                });
                vertices.push(Vertex2d {
                    position: br.into(),
                    uv: [sprite.max.x, sprite.max.y],
                    color: [1.0; 4],
                });
                vertices.push(Vertex2d {
                    position: tl.into(),
                    uv: [sprite.min.x, sprite.min.y],
                    color: [1.0; 4],
                });
                vertices.push(Vertex2d {
                    position: br.into(),
                    uv: [sprite.max.x, sprite.max.y],
                    color: [1.0; 4],
                });
                vertices.push(Vertex2d {
                    position: tr.into(),
                    uv: [sprite.max.x, sprite.min.y],
                    color: [1.0; 4],
                });
            }

            let vertex_buffer = ctx
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("sprite_batch_vertex"),
                    contents: cast_slice(&vertices),
                    usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::VERTEX,
                });

            buffers.push(vertex_buffer);

            if !self.bind_groups.contains_key(id) {
                let sampler = ctx
                    .device
                    .create_sampler(&wgpu::SamplerDescriptor::default());

                let layout =
                    ctx.device
                        .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                            label: Some("2d_pass_layout"),
                            entries: &[
                                wgpu::BindGroupLayoutEntry {
                                    binding: 0,
                                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                                    ty: wgpu::BindingType::Texture {
                                        sample_type: wgpu::TextureSampleType::Float {
                                            filterable: true,
                                        },
                                        view_dimension: wgpu::TextureViewDimension::D2,
                                        multisampled: false,
                                    },
                                    count: None,
                                },
                                wgpu::BindGroupLayoutEntry {
                                    binding: 1,
                                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                                    ty: wgpu::BindingType::Sampler {
                                        filtering: true,
                                        comparison: false,
                                    },
                                    count: None,
                                },
                            ],
                        });

                let bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("2d_pass_bind_group"),
                    layout: &layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(
                                &sprites.first().unwrap().view,
                            ),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(&sampler),
                        },
                    ],
                });

                self.bind_groups.insert(*id, bind_group);
            }
        }

        let mut buffers = buffers.iter();

        for (id, sprites) in sprites.batches {
            render_pass.set_bind_group(0, self.bind_groups.get(&id).unwrap(), &[]);
            render_pass.set_vertex_buffer(0, buffers.next().unwrap().slice(..));

            render_pass.draw(0..sprites.len() as u32 * 6, 0..1);
        }

        drop(render_pass);

        ctx.queue.submit(std::iter::once(encoder.finish()));
    }
}
