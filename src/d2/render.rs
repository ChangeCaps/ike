use super::{
    sprite::{Sprite, Sprites},
    transform2d::Transform2d,
};
use crate::{
    id::{HasId, Id},
    prelude::Font,
    renderer::{PassNode, PassNodeCtx, RenderCtx, SampleCount},
    texture::Texture,
};
use bytemuck::{cast_slice, Pod, Zeroable};
use glam::{Vec2, Vec3};
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
    pub fn draw_text(&mut self, font: &Font, text: &str, transform: &Transform2d, size: f32) {
        let mut height = 0.0f32;
        let mut width = 0.0;

        for c in text.chars() {
            let glyph = if let Some(glyph) = font.raw_glyph(c) {
                glyph
            } else {
                continue;
            };

            width += glyph.width() as f32 / font.texture.width() as f32;
            height = height.max(glyph.height() as f32);
        }

        let texture = font.texture.texture(self.render_ctx);

        let mut x = -width / 2.0;

        for c in text.chars() {
            let glyph = if let Some(glyph) = font.raw_glyph(c) {
                glyph
            } else {
                continue;
            };

            let mut transform = transform.clone();
            transform.translation.x += x;

            let sprite = Sprite {
                transform: transform.matrix(),
                depth: 0.0,
                width: glyph.width() as f32 / height * size,
                height: glyph.height() as f32 / height * size,
                min: glyph.min.as_f32() / font.texture.size().as_f32(),
                max: glyph.max.as_f32() / font.texture.size().as_f32(),
                texture_id: font.texture.id(),
                view: texture.create_view(&Default::default()),
            };

            self.sprites.draw(sprite);

            x += glyph.width() as f32 / height * size;
        }
    }

    #[inline]
    pub fn draw_text_depth(
        &mut self,
        font: &Font,
        text: &str,
        transform: &Transform2d,
        size: f32,
        depth: f32,
    ) {
        let mut height = 0.0f32;
        let mut width = 0.0;

        for c in text.chars() {
            let glyph = if let Some(glyph) = font.raw_glyph(c) {
                glyph
            } else {
                continue;
            };

            width += glyph.width() as f32 / font.texture.width() as f32;
            height = height.max(glyph.height() as f32);
        }

        let texture = font.texture.texture(self.render_ctx);

        let mut x = -width / 2.0;

        for c in text.chars() {
            let glyph = if let Some(glyph) = font.raw_glyph(c) {
                glyph
            } else {
                continue;
            };

            let mut transform = transform.clone();
            transform.translation.x += x;

            let sprite = Sprite {
                transform: transform.matrix(),
                depth,
                width: glyph.width() as f32 / height * size,
                height: glyph.height() as f32 / height * size,
                min: glyph.min.as_f32() / font.texture.size().as_f32(),
                max: glyph.max.as_f32() / font.texture.size().as_f32(),
                texture_id: font.texture.id(),
                view: texture.create_view(&Default::default()),
            };

            self.sprites.draw(sprite);

            x += glyph.width() as f32 / height * size;
        }
    }

    #[inline]
    pub fn draw_texture(&mut self, texture: &Texture, transform: &Transform2d) {
        let view = texture
            .texture(self.render_ctx)
            .create_view(&Default::default());

        let sprite = Sprite {
            transform: transform.matrix(),
            width: texture.width() as f32,
            height: texture.height() as f32,
            depth: 0.0,
            min: Vec2::ZERO,
            max: Vec2::ONE,
            texture_id: texture.id(),
            view,
        };

        self.sprites.draw(sprite);
    }

    #[inline]
    pub fn draw_texture_depth(&mut self, texture: &Texture, transform: &Transform2d, depth: f32) {
        let view = texture
            .texture(self.render_ctx)
            .create_view(&Default::default());

        let sprite = Sprite {
            transform: transform.matrix(),
            width: texture.width() as f32,
            height: texture.height() as f32,
            depth,
            min: Vec2::ZERO,
            max: Vec2::ONE,
            texture_id: texture.id(),
            view,
        };

        self.sprites.draw(sprite);
    }

    #[inline]
    pub fn draw_texture_offset(
        &mut self,
        texture: &Texture,
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

struct Draw {
    id: Id<Texture>,
    vertex_count: u32,
    vertices: wgpu::Buffer,
}

pub struct SpriteNode2d {
    draws: Vec<Draw>,
    bind_groups: HashMap<Id<Texture>, wgpu::BindGroup>,
    pipelines: HashMap<wgpu::TextureFormat, wgpu::RenderPipeline>,
}

impl Default for SpriteNode2d {
    #[inline]
    fn default() -> Self {
        Self {
            draws: Vec::new(),
            bind_groups: Default::default(),
            pipelines: Default::default(),
        }
    }
}

impl SpriteNode2d {
    #[inline]
    pub fn new() -> Self {
        Default::default()
    }
}

impl<S: Render2d> PassNode<S> for SpriteNode2d {
    #[inline]
    fn run<'a>(&'a mut self, ctx: &mut PassNodeCtx<'_, 'a>, state: &mut S) {
        let sample_count = ctx.data.get::<SampleCount>().unwrap().0;
        let pipeline = self
            .pipelines
            .entry(ctx.view.format)
            .or_insert_with(|| crate_pipeline(ctx.render_ctx, ctx.view.format, sample_count));

        let mut sprites = Sprites::default();

        let mut render_ctx = Render2dCtx {
            sprites: &mut sprites,
            render_ctx: ctx.render_ctx,
        };

        state.render(&mut render_ctx);

        self.draws.clear();

        ctx.render_pass.set_pipeline(pipeline);

        struct SpriteDraw<'a> {
            id: Id<Texture>,
            depth: f32,
            vertices: [Vertex2d; 6],
            view: &'a wgpu::TextureView,
        }

        let mut sprites = sprites
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

                let bl = ctx.view.view_proj.transform_point3(bl.extend(sprite.depth));
                let tl = ctx.view.view_proj.transform_point3(tl.extend(sprite.depth));
                let br = ctx.view.view_proj.transform_point3(br.extend(sprite.depth));
                let tr = ctx.view.view_proj.transform_point3(tr.extend(sprite.depth));

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
                        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("sprite_batch_vertex"),
                            contents: cast_slice(&vertices),
                            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::VERTEX,
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
                    .create_sampler(&wgpu::SamplerDescriptor::default());

                let layout = ctx.render_ctx.device.create_bind_group_layout(
                    &wgpu::BindGroupLayoutDescriptor {
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
                    },
                );

                let bind_group =
                    ctx.render_ctx
                        .device
                        .create_bind_group(&wgpu::BindGroupDescriptor {
                            label: Some("2d_pass_bind_group"),
                            layout: &layout,
                            entries: &[
                                wgpu::BindGroupEntry {
                                    binding: 0,
                                    resource: wgpu::BindingResource::TextureView(sprite_draw.view),
                                },
                                wgpu::BindGroupEntry {
                                    binding: 1,
                                    resource: wgpu::BindingResource::Sampler(&sampler),
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
