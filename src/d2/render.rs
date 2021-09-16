use super::{
    sprite::{BatchedSprite, Sprites},
    transform2d::Transform2d,
};
use crate::{
    id::{HasId, Id},
    prelude::{Camera, Font},
    renderer::{Drawable, PassNode, PassNodeCtx, RenderCtx, SampleCount, TargetFormat, ViewProj},
    texture::Texture,
};
use bytemuck::{cast_slice, Pod, Zeroable};
use glam::{Mat3, Mat4, Vec2};
use std::collections::HashMap;

fn create_pipeline(
    ctx: &RenderCtx,
    format: ike_wgpu::TextureFormat,
    sample_count: u32,
) -> ike_wgpu::RenderPipeline {
    let shader_module = ctx
        .device
        .create_shader_module(&ike_wgpu::include_wgsl!("shader.wgsl"));

    let bind_group_layout =
        ctx.device
            .create_bind_group_layout(&ike_wgpu::BindGroupLayoutDescriptor {
                label: Some("2d_bind_group_layout"),
                entries: &[
                    ike_wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ike_wgpu::ShaderStages::VERTEX_FRAGMENT,
                        ty: ike_wgpu::BindingType::Texture {
                            sample_type: ike_wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: ike_wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    ike_wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ike_wgpu::ShaderStages::VERTEX_FRAGMENT,
                        ty: ike_wgpu::BindingType::Sampler {
                            filtering: true,
                            comparison: false,
                        },
                        count: None,
                    },
                ],
            });

    let layout = ctx
        .device
        .create_pipeline_layout(&ike_wgpu::PipelineLayoutDescriptor {
            label: Some("2d_pipeline_layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

    let pipeline = ctx
        .device
        .create_render_pipeline(&ike_wgpu::RenderPipelineDescriptor {
            label: Some("2d_pipeline"),
            layout: Some(&layout),
            vertex: ike_wgpu::VertexState {
                module: &shader_module,
                buffers: &[ike_wgpu::VertexBufferLayout {
                    array_stride: 36,
                    step_mode: ike_wgpu::VertexStepMode::Vertex,
                    attributes: &[
                        ike_wgpu::VertexAttribute {
                            format: ike_wgpu::VertexFormat::Float32x3,
                            offset: 0,
                            shader_location: 0,
                        },
                        ike_wgpu::VertexAttribute {
                            format: ike_wgpu::VertexFormat::Float32x2,
                            offset: 12,
                            shader_location: 1,
                        },
                        ike_wgpu::VertexAttribute {
                            format: ike_wgpu::VertexFormat::Float32x4,
                            offset: 20,
                            shader_location: 2,
                        },
                    ],
                }],
                entry_point: "main",
            },
            fragment: Some(ike_wgpu::FragmentState {
                module: &shader_module,
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

pub struct TextSprite<'a> {
    pub transform: Transform2d,
    pub depth: f32,
    pub size: f32,
    pub text: &'a str,
    pub font: &'a Font,
}

impl<'a> TextSprite<'a> {
    #[inline]
    pub fn new(font: &'a Font, transform: Transform2d) -> Self {
        Self {
            transform,
            depth: 0.0,
            size: 10.0,
            text: "",
            font,
        }
    }
}

impl Drawable for TextSprite<'_> {
    type Node = SpriteNode2d;

    #[inline]
    fn draw(&self, ctx: &RenderCtx, node: &mut Self::Node) {
        let mut height = 0.0f32;
        let mut width = 0.0;

        for c in self.text.chars() {
            let glyph = if let Some(glyph) = self.font.raw_glyph(c) {
                glyph
            } else {
                continue;
            };

            width += glyph.width() as f32 / self.font.texture.width() as f32;
            height = height.max(glyph.height() as f32);
        }

        let texture = self.font.texture.texture(ctx);

        let mut x = -width / 2.0;

        for c in self.text.chars() {
            let glyph = if let Some(glyph) = self.font.raw_glyph(c) {
                glyph
            } else {
                continue;
            };

            let mut transform = self.transform.clone();
            transform.translation.x += x;

            let sprite = BatchedSprite {
                transform: transform.matrix(),
                depth: self.depth,
                width: glyph.width() as f32 / height * self.size,
                height: glyph.height() as f32 / height * self.size,
                min: glyph.min.as_f32() / self.font.texture.size().as_f32(),
                max: glyph.max.as_f32() / self.font.texture.size().as_f32(),
                texture_id: self.font.texture.id(),
                view: texture.create_view(&Default::default()),
            };

            node.sprites.draw(sprite);

            x += glyph.width() as f32 / height * self.size;
        }
    }
}

pub struct Sprite<'a> {
    pub transform: Transform2d,
    pub depth: f32,
    pub width: f32,
    pub height: f32,
    pub min: Vec2,
    pub max: Vec2,
    pub texture: &'a Texture,
}

impl<'a> Sprite<'a> {
    #[inline]
    pub fn new(texture: &'a Texture, transform: Transform2d) -> Self {
        Self {
            transform: transform.clone(),
            depth: 0.0,
            width: texture.width() as f32,
            height: texture.height() as f32,
            min: Vec2::ZERO,
            max: Vec2::ONE,
            texture,
        }
    }

    #[inline]
    pub fn offset(&mut self, offset: Vec2) {
        self.transform.translation += offset;
    }
}

impl Drawable for Sprite<'_> {
    type Node = SpriteNode2d;

    #[inline]
    fn draw(&self, ctx: &RenderCtx, node: &mut Self::Node) {
        let sprite = BatchedSprite {
            transform: self.transform.matrix(),
            depth: self.depth,
            width: self.width,
            height: self.height,
            min: self.min,
            max: self.max,
            texture_id: self.texture.id(),
            view: self.texture.texture(ctx).create_view(&Default::default()),
        };

        node.sprites.draw(sprite);
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Vertex2d {
    position: [f32; 3],
    uv: [f32; 2],
    color: [f32; 4],
}

struct Draw {
    id: Id<Texture>,
    vertex_count: u32,
    vertices: ike_wgpu::Buffer,
}

pub struct SpriteNode2d {
    draws: Vec<Draw>,
    bind_groups: HashMap<Id<Texture>, ike_wgpu::BindGroup>,
    pipelines: HashMap<ike_wgpu::TextureFormat, ike_wgpu::RenderPipeline>,
    sprites: Sprites,
}

impl Default for SpriteNode2d {
    #[inline]
    fn default() -> Self {
        Self {
            draws: Vec::new(),
            bind_groups: Default::default(),
            pipelines: Default::default(),
            sprites: Default::default(),
        }
    }
}

impl SpriteNode2d {
    #[inline]
    pub fn new() -> Self {
        Default::default()
    }
}

impl<S> PassNode<S> for SpriteNode2d {
    #[inline]
    fn clear(&mut self) {
        self.sprites.batches.clear();
    }

    #[inline]
    fn run<'a>(&'a mut self, ctx: &mut PassNodeCtx<'_, 'a>, _: &mut S) {
        let sample_count = ctx.data.get::<SampleCount>().unwrap().0;
        let format = ctx
            .data
            .get::<TargetFormat>()
            .cloned()
            .unwrap_or_else(|| TargetFormat(ctx.view.format))
            .0;
        let camera = ctx.data.get::<Camera>().unwrap_or_else(|| &ctx.view.camera);

        let view_proj = camera.view_proj();

        let pipeline = self
            .pipelines
            .entry(format)
            .or_insert_with(|| create_pipeline(ctx.render_ctx, format, sample_count));

        self.draws.clear();

        ctx.render_pass.set_pipeline(pipeline);

        struct SpriteDraw<'a> {
            id: Id<Texture>,
            depth: f32,
            vertices: [Vertex2d; 6],
            view: &'a ike_wgpu::TextureView,
        }

        let mut sprites = self
            .sprites
            .batches
            .values()
            .flatten()
            .map(|sprite| {
                let w = sprite.width / 2.0;
                let h = sprite.height / 2.0;

                let bl = Vec2::new(-w, -h);
                let tl = Vec2::new(-w, h);
                let br = Vec2::new(w, -h);
                let tr = Vec2::new(w, h);

                let bl = sprite.transform.transform_point2(bl);
                let tl = sprite.transform.transform_point2(tl);
                let br = sprite.transform.transform_point2(br);
                let tr = sprite.transform.transform_point2(tr);

                let bl = view_proj.transform_point3(bl.extend(sprite.depth));
                let tl = view_proj.transform_point3(tl.extend(sprite.depth));
                let br = view_proj.transform_point3(br.extend(sprite.depth));
                let tr = view_proj.transform_point3(tr.extend(sprite.depth));

                // calculate average depth
                let depth = (bl.z + tl.z + br.z + tr.z) / 4.0;

                SpriteDraw {
                    id: sprite.texture_id,
                    depth,
                    vertices: [
                        Vertex2d {
                            position: bl.into(),
                            uv: [sprite.min.x, sprite.max.y],
                            color: [1.0; 4],
                        },
                        Vertex2d {
                            position: tl.into(),
                            uv: [sprite.min.x, sprite.min.y],
                            color: [1.0; 4],
                        },
                        Vertex2d {
                            position: br.into(),
                            uv: [sprite.max.x, sprite.max.y],
                            color: [1.0; 4],
                        },
                        Vertex2d {
                            position: tl.into(),
                            uv: [sprite.min.x, sprite.min.y],
                            color: [1.0; 4],
                        },
                        Vertex2d {
                            position: br.into(),
                            uv: [sprite.max.x, sprite.max.y],
                            color: [1.0; 4],
                        },
                        Vertex2d {
                            position: tr.into(),
                            uv: [sprite.max.x, sprite.min.y],
                            color: [1.0; 4],
                        },
                    ],
                    view: &sprite.view,
                }
            })
            .collect::<Vec<_>>();

        sprites.sort_by(|a, b| b.depth.partial_cmp(&a.depth).unwrap());

        let mut current_id: Option<Id<Texture>> = None;
        let mut vertices: Vec<Vertex2d> = Vec::new();

        let draw = |draws: &mut Vec<Draw>,
                    ctx: &mut PassNodeCtx,
                    vertices: &[Vertex2d],
                    current_id: &Option<Id<Texture>>| {
            if let Some(current_id) = current_id {
                let vertex_buffer =
                    ctx.render_ctx
                        .device
                        .create_buffer_init(&ike_wgpu::BufferInitDescriptor {
                            label: Some("sprite_batch_vertex"),
                            contents: cast_slice(&vertices),
                            usage: ike_wgpu::BufferUsages::COPY_DST
                                | ike_wgpu::BufferUsages::VERTEX,
                        });

                draws.push(Draw {
                    id: *current_id,
                    vertices: vertex_buffer,
                    vertex_count: vertices.len() as u32,
                });
            }
        };

        for sprite_draw in sprites {
            if !self.bind_groups.contains_key(&sprite_draw.id) {
                let sampler = ctx
                    .render_ctx
                    .device
                    .create_sampler(&ike_wgpu::SamplerDescriptor::default());

                let layout = ctx.render_ctx.device.create_bind_group_layout(
                    &ike_wgpu::BindGroupLayoutDescriptor {
                        label: Some("2d_pass_layout"),
                        entries: &[
                            ike_wgpu::BindGroupLayoutEntry {
                                binding: 0,
                                visibility: ike_wgpu::ShaderStages::VERTEX_FRAGMENT,
                                ty: ike_wgpu::BindingType::Texture {
                                    sample_type: ike_wgpu::TextureSampleType::Float {
                                        filterable: true,
                                    },
                                    view_dimension: ike_wgpu::TextureViewDimension::D2,
                                    multisampled: false,
                                },
                                count: None,
                            },
                            ike_wgpu::BindGroupLayoutEntry {
                                binding: 1,
                                visibility: ike_wgpu::ShaderStages::VERTEX_FRAGMENT,
                                ty: ike_wgpu::BindingType::Sampler {
                                    filtering: true,
                                    comparison: false,
                                },
                                count: None,
                            },
                        ],
                    },
                );

                let bind_group =
                    ctx.render_ctx
                        .device
                        .create_bind_group(&ike_wgpu::BindGroupDescriptor {
                            label: Some("2d_pass_bind_group"),
                            layout: &layout,
                            entries: &[
                                ike_wgpu::BindGroupEntry {
                                    binding: 0,
                                    resource: ike_wgpu::BindingResource::TextureView(
                                        sprite_draw.view,
                                    ),
                                },
                                ike_wgpu::BindGroupEntry {
                                    binding: 1,
                                    resource: ike_wgpu::BindingResource::Sampler(&sampler),
                                },
                            ],
                        });

                self.bind_groups.insert(sprite_draw.id, bind_group);
            }

            if current_id != Some(sprite_draw.id) {
                draw(&mut self.draws, ctx, &vertices, &current_id);

                vertices.clear();
                current_id = Some(sprite_draw.id);
            }

            vertices.push(sprite_draw.vertices[0]);
            vertices.push(sprite_draw.vertices[1]);
            vertices.push(sprite_draw.vertices[2]);
            vertices.push(sprite_draw.vertices[3]);
            vertices.push(sprite_draw.vertices[4]);
            vertices.push(sprite_draw.vertices[5]);
        }

        if current_id.is_some() {
            draw(&mut self.draws, ctx, &vertices, &current_id);
        }

        for draw in &self.draws {
            ctx.render_pass
                .set_bind_group(0, self.bind_groups.get(&draw.id).unwrap(), &[]);
            ctx.render_pass
                .set_vertex_buffer(0, draw.vertices.slice(..));

            ctx.render_pass.draw(0..draw.vertex_count, 0..1);
        }
    }
}
