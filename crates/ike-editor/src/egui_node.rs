use std::collections::HashMap;

use bytemuck::{cast_slice, Pod, Zeroable};
use ike::{
    egui::{ClippedMesh, Event, Modifiers, Pos2, Rect, Rgba, TextureId},
    prelude::*,
};

#[inline]
pub fn key_w2e(w: Key) -> Option<ike::egui::Key> {
    use ike::egui::Key as E;
    use ike::prelude::Key as W;
    Some(match w {
        W::Down => E::ArrowDown,
        W::Left => E::ArrowLeft,
        W::Right => E::ArrowRight,
        W::Up => E::ArrowUp,

        W::Escape => E::Escape,
        W::Tab => E::Tab,
        W::Back => E::Backspace,
        W::Return => E::Enter,
        W::Space => E::Insert,
        W::Delete => E::Delete,
        W::Home => E::Home,
        W::End => E::End,
        W::PageUp => E::PageUp,
        W::PageDown => E::PageDown,

        W::Key1 | W::Numpad1 => E::Num1,
        W::Key2 | W::Numpad2 => E::Num2,
        W::Key3 | W::Numpad3 => E::Num3,
        W::Key4 | W::Numpad4 => E::Num4,
        W::Key5 | W::Numpad5 => E::Num5,
        W::Key6 | W::Numpad6 => E::Num6,
        W::Key7 | W::Numpad7 => E::Num7,
        W::Key8 | W::Numpad8 => E::Num8,
        W::Key9 | W::Numpad9 => E::Num9,
        W::Key0 | W::Numpad0 => E::Num0,

        W::A => E::A,
        W::B => E::B,
        W::C => E::C,
        W::D => E::D,
        W::E => E::E,
        W::F => E::F,
        W::G => E::G,
        W::H => E::H,
        W::I => E::I,
        W::J => E::J,
        W::K => E::K,
        W::L => E::L,
        W::M => E::M,
        W::N => E::N,
        W::O => E::O,
        W::P => E::P,
        W::Q => E::Q,
        W::R => E::R,
        W::S => E::S,
        W::T => E::T,
        W::U => E::U,
        W::V => E::V,
        W::W => E::W,
        W::X => E::X,
        W::Y => E::Y,
        W::Z => E::Z,

        _ => {
            return None;
        }
    })
}

#[inline]
pub fn mouse_w2e(mouse_button: MouseButton) -> Option<ike::egui::PointerButton> {
    use ike::egui::PointerButton as E;
    use MouseButton as W;

    match mouse_button {
        W::Left => Some(E::Primary),
        W::Right => Some(E::Secondary),
        W::Middle => Some(E::Middle),
        _ => None,
    }
}

impl AppState {
    pub fn egui_input(&mut self, ctx: &mut UpdateCtx) {
        let modifiers = Modifiers {
            alt: ctx.key_input.down(&Key::LAlt),
            ctrl: ctx.key_input.down(&Key::LControl),
            shift: ctx.key_input.down(&Key::LShift),
            mac_cmd: ctx.key_input.down(&Key::LWin),
            command: ctx.key_input.down(&Key::LWin),
        };

        for key in ctx.key_input.iter_pressed() {
            if let Some(key) = key_w2e(*key) {
                self.raw_input.events.push(Event::Key {
                    key,
                    pressed: true,
                    modifiers,
                });
            }
        }

        for key in ctx.key_input.iter_released() {
            if let Some(key) = key_w2e(*key) {
                self.raw_input.events.push(Event::Key {
                    key,
                    pressed: false,
                    modifiers,
                });
            }
        }

        let pos = ike::egui::Pos2::new(ctx.mouse.position.x, ctx.mouse.position.y);

        for button in ctx.mouse_input.iter_pressed() {
            if let Some(button) = mouse_w2e(*button) {
                self.raw_input.events.push(Event::PointerButton {
                    pos,
                    button,
                    pressed: true,
                    modifiers,
                });
            }
        }

        for button in ctx.mouse_input.iter_released() {
            if let Some(button) = mouse_w2e(*button) {
                self.raw_input.events.push(Event::PointerButton {
                    pos,
                    button,
                    pressed: false,
                    modifiers,
                });
            }
        }

        for c in ctx.char_input {
            if !c.is_control() {
                self.raw_input.events.push(Event::Text(c.to_string()));
            }
        }

        self.raw_input.events.push(Event::PointerMoved(Pos2::new(
            ctx.mouse.position.x,
            ctx.mouse.position.y,
        )));
    }
}

use crate::AppState;

fn pipeline(ctx: &RenderCtx, format: wgpu::TextureFormat) -> wgpu::RenderPipeline {
    let shader_module = ctx
        .device
        .create_shader_module(&wgpu::include_wgsl!("shader.wgsl"));

    let size_layout = ctx
        .device
        .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("egui_bind_group_layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

    let texture_layout = ctx
        .device
        .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("egui_texture_bind_layout"),
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
            label: Some("egui_pipeline_layout"),
            bind_group_layouts: &[&size_layout, &texture_layout],
            push_constant_ranges: &[],
        });

    let pipeline = ctx
        .device
        .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("egui_pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader_module,
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: 32,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x2,
                            offset: 0,
                            shader_location: 0,
                        },
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x2,
                            offset: 8,
                            shader_location: 1,
                        },
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x4,
                            offset: 16,
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
            multisample: Default::default(),
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

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Vertex {
    position: [f32; 2],
    uv: [f32; 2],
    color: [f32; 4],
}

#[derive(Default)]
pub struct EguiNode {
    size_buffer: Option<wgpu::Buffer>,
    size_bind_group: Option<wgpu::BindGroup>,
    egui_texture: Option<wgpu::Texture>,
    egui_texture_bind_group: Option<wgpu::BindGroup>,
    buffers: Vec<(wgpu::Buffer, wgpu::Buffer)>,
    texture_bind_groups: HashMap<u64, (wgpu::BindGroup, TextureVersion)>,
    pipelines: HashMap<wgpu::TextureFormat, wgpu::RenderPipeline>,
}

fn texture_bind_group(ctx: &RenderCtx, view: &wgpu::TextureView) -> wgpu::BindGroup {
    let sampler = ctx.device.create_sampler(&wgpu::SamplerDescriptor {
        min_filter: wgpu::FilterMode::Linear,
        mag_filter: wgpu::FilterMode::Linear,
        ..Default::default()
    });

    let layout = ctx
        .device
        .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("egui_texture_bind_layout"),
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

    let bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("egui_texture_bind"),
        layout: &layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
        ],
    });

    bind_group
}

impl PassNode<AppState> for EguiNode {
    fn run<'a>(&'a mut self, ctx: &mut ike::renderer::PassNodeCtx<'_, 'a>, state: &mut AppState) {
        state.raw_input.screen_rect = Some(Rect::from_min_size(
            Pos2::ZERO,
            ike::egui::Vec2::new(ctx.view.width as f32, ctx.view.height as f32),
        ));

        let (_output, shapes) = state.egui_ctx.end_frame();
        let meshes = state.egui_ctx.tessellate(shapes);

        // create egui texture
        if self.egui_texture.is_none() {
            let texture = state.egui_ctx.texture();

            let data = texture
                .srgba_pixels()
                .flat_map(|pixel| [pixel.r(), pixel.g(), pixel.b(), pixel.a()])
                .collect::<Vec<_>>();

            let texture = ctx.render_ctx.device.create_texture_with_data(
                &ctx.render_ctx.queue,
                &wgpu::TextureDescriptor {
                    label: Some("egui_texture"),
                    size: wgpu::Extent3d {
                        width: texture.width as u32,
                        height: texture.height as u32,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::Rgba8UnormSrgb,
                    usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                },
                &data,
            );

            let view = texture.create_view(&Default::default());

            let bind_group = texture_bind_group(ctx.render_ctx, &view);

            self.egui_texture = Some(texture);
            self.egui_texture_bind_group = Some(bind_group);
        }

        // create size buffer
        if let Some(ref buffer) = self.size_buffer {
            ctx.render_ctx.queue.write_buffer(
                buffer,
                0,
                cast_slice(&[ctx.view.width as f32, ctx.view.height as f32]),
            );
        } else {
            let buffer = ctx
                .render_ctx
                .device
                .create_buffer_init(&wgpu::BufferInitDescriptor {
                    label: Some("size_buffer"),
                    contents: cast_slice(&[ctx.view.width as f32, ctx.view.height as f32]),
                    usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
                });

            let layout =
                ctx.render_ctx
                    .device
                    .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                        label: Some("egui_bind_group_layout"),
                        entries: &[wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        }],
                    });

            let bind_group = ctx
                .render_ctx
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("egui_size_bind_group"),
                    layout: &layout,
                    entries: &[wgpu::BindGroupEntry {
                        binding: 0,
                        resource: buffer.as_entire_binding(),
                    }],
                });

            self.size_buffer = Some(buffer);
            self.size_bind_group = Some(bind_group);
        }

        // prepare buffers
        self.buffers.clear();

        for ClippedMesh(_rect, mesh) in &meshes {
            let mut vertices = Vec::new();

            for vertex in &mesh.vertices {
                let rgba = Rgba::from(vertex.color);

                vertices.push(Vertex {
                    position: vertex.pos.into(),
                    uv: vertex.uv.into(),
                    color: [rgba.r(), rgba.g(), rgba.b(), rgba.a()],
                });
            }

            let vertex_buffer =
                ctx.render_ctx
                    .device
                    .create_buffer_init(&wgpu::BufferInitDescriptor {
                        label: Some("egui_vertex_buffer"),
                        contents: cast_slice(&vertices),
                        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::VERTEX,
                    });

            let index_buffer =
                ctx.render_ctx
                    .device
                    .create_buffer_init(&wgpu::BufferInitDescriptor {
                        label: Some("egui_index_buffer"),
                        contents: cast_slice(&mesh.indices),
                        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::INDEX,
                    });

            self.buffers.push((vertex_buffer, index_buffer));

            // prepare texture bind_groups
            if let TextureId::User(ref id) = mesh.texture_id {
                let texture = state.textures.get(&Id::from(*id)).unwrap();

                let recreate = if let Some((_, version)) =
                    self.texture_bind_groups.get(&texture.id().inner())
                {
                    texture.outdated(*version)
                } else {
                    true
                };

                if recreate {
                    let view = texture
                        .texture(ctx.render_ctx)
                        .create_view(&Default::default());

                    let bind_group = texture_bind_group(ctx.render_ctx, &view);
                    self.texture_bind_groups
                        .insert(*id, (bind_group, texture.version()));
                }
            }
        }

        // prepare pipeline
        let pipeline = self
            .pipelines
            .entry(ctx.view.format)
            .or_insert_with(|| pipeline(ctx.render_ctx, ctx.view.format));

        ctx.render_pass.set_pipeline(pipeline);

        ctx.render_pass
            .set_bind_group(0, self.size_bind_group.as_ref().unwrap(), &[]);

        let mut buffers = self.buffers.iter();

        for ClippedMesh(_rect, mesh) in meshes {
            // TODO(changecaps): set scissor rect

            // retrieve prepared buffers
            let (vertex_buffer, index_buffer) = buffers.next().unwrap();

            ctx.render_pass
                .set_vertex_buffer(0, vertex_buffer.slice(..));
            ctx.render_pass
                .set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);

            // set texture bind group
            match mesh.texture_id {
                TextureId::User(id) => {
                    ctx.render_pass.set_bind_group(
                        1,
                        &self.texture_bind_groups.get(&id).unwrap().0,
                        &[],
                    );
                }
                TextureId::Egui => {
                    ctx.render_pass.set_bind_group(
                        1,
                        self.egui_texture_bind_group.as_ref().unwrap(),
                        &[],
                    );
                }
            }

            // draw
            ctx.render_pass
                .draw_indexed(0..mesh.indices.len() as u32, 0, 0..1);
        }
    }
}
