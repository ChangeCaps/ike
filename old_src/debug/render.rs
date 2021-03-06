use std::collections::HashMap;

use bytemuck::cast_slice;
use glam::{Mat4, Vec2, Vec3, Vec4Swizzles};

use crate::{
    buffer::Buffer,
    prelude::{Camera, Color},
    render_texture::RenderTexture,
    renderer::{render_device, EdgeSlotInfo, GraphError, NodeEdge, RenderCtx, RenderNode},
    transform::Transform,
};

fn create_pipeline(format: ike_wgpu::TextureFormat, sample_count: u32) -> ike_wgpu::RenderPipeline {
    let device = render_device();

    let module = device.create_shader_module(&ike_wgpu::include_wgsl!("shader.wgsl"));

    device.create_render_pipeline(&ike_wgpu::RenderPipelineDescriptor {
        label: Some("debug_line_pipeline"),
        layout: None,
        vertex: ike_wgpu::VertexState {
            module: &module,
            entry_point: "main",
            buffers: &[ike_wgpu::VertexBufferLayout {
                array_stride: std::mem::size_of::<RawDebugLine>() as u64,
                step_mode: ike_wgpu::VertexStepMode::Instance,
                attributes: &[
                    ike_wgpu::VertexAttribute {
                        offset: 0,
                        format: ike_wgpu::VertexFormat::Float32x4,
                        shader_location: 0,
                    },
                    ike_wgpu::VertexAttribute {
                        offset: 16,
                        format: ike_wgpu::VertexFormat::Float32x4,
                        shader_location: 1,
                    },
                    ike_wgpu::VertexAttribute {
                        offset: 32,
                        format: ike_wgpu::VertexFormat::Float32x4,
                        shader_location: 2,
                    },
                    ike_wgpu::VertexAttribute {
                        offset: 48,
                        format: ike_wgpu::VertexFormat::Float32x4,
                        shader_location: 3,
                    },
                    ike_wgpu::VertexAttribute {
                        offset: 64,
                        format: ike_wgpu::VertexFormat::Float32x4,
                        shader_location: 4,
                    },
                    ike_wgpu::VertexAttribute {
                        offset: 80,
                        format: ike_wgpu::VertexFormat::Float32x2,
                        shader_location: 5,
                    },
                ],
            }],
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
            depth_write_enabled: true,
            depth_compare: ike_wgpu::CompareFunction::LessEqual,
            stencil: Default::default(),
            bias: Default::default(),
        }),
    })
}

#[derive(Clone, Debug)]
pub struct DebugLine {
    pub from: Vec3,
    pub to: Vec3,
    pub width: f32,
    pub color: Color,
    pub depth: bool,
}

impl DebugLine {
    #[inline]
    pub fn new(from: Vec3, to: Vec3) -> Self {
        Self {
            from,
            to,
            width: 0.002,
            color: Color::WHITE,
            depth: false,
        }
    }

    #[inline]
    pub fn color(from: Vec3, to: Vec3, color: Color) -> Self {
        Self {
            color,
            ..Self::new(from, to)
        }
    }
}

/*
#[derive(Clone, Debug)]
pub struct DebugMesh<'a> {
    pub mesh: &'a Mesh,
    pub transform: Option<&'a Transform>,
    pub vertex_normals: Option<Color>,
    pub face_normals: Option<Color>,
    pub faces: Option<Color>,
    pub width: f32,
    pub depth: bool,
}

impl<'a> DebugMesh<'a> {
    #[inline]
    pub fn new(mesh: &'a Mesh) -> Self {
        Self {
            mesh,
            transform: None,
            vertex_normals: None,
            face_normals: None,
            faces: Some(Color::WHITE),
            width: 0.002,
            depth: false,
        }
    }

    #[inline]
    pub fn with_transform(mesh: &'a Mesh, transform: &'a Transform) -> Self {
        Self {
            transform: Some(transform),
            ..Self::new(mesh)
        }
    }
}
*/

/*
impl Drawable for DebugMesh<'_> {
    type Node = &'static mut DebugNode;

    #[inline]
    fn draw(&self, _ctx: &RenderCtx, node: &mut DebugNode) {
        let transform = if let Some(transform) = self.transform {
            transform.matrix()
        } else {
            Mat4::IDENTITY
        };

        if let Some(color) = self.faces {
            for i in 0..self.mesh.indices.len() / 3 {
                let i0 = self.mesh.indices[i * 3 + 0];
                let i1 = self.mesh.indices[i * 3 + 1];
                let i2 = self.mesh.indices[i * 3 + 2];

                let v0 = self.mesh.vertices[i0 as usize];
                let v1 = self.mesh.vertices[i1 as usize];
                let v2 = self.mesh.vertices[i2 as usize];

                node.lines.push(DebugLine {
                    from: transform.transform_point3(v0.position),
                    to: transform.transform_point3(v1.position),
                    width: self.width,
                    color,
                    depth: self.depth,
                });

                node.lines.push(DebugLine {
                    from: transform.transform_point3(v1.position),
                    to: transform.transform_point3(v2.position),
                    width: self.width,
                    color,
                    depth: self.depth,
                });

                node.lines.push(DebugLine {
                    from: transform.transform_point3(v2.position),
                    to: transform.transform_point3(v0.position),
                    width: self.width,
                    color,
                    depth: self.depth,
                });
            }
        }

        if let Some(color) = self.face_normals {
            for i in 0..self.mesh.indices.len() / 3 {
                let i0 = self.mesh.indices[i * 3 + 0];
                let i1 = self.mesh.indices[i * 3 + 1];
                let i2 = self.mesh.indices[i * 3 + 2];

                let v0 = self.mesh.vertices[i0 as usize];
                let v1 = self.mesh.vertices[i1 as usize];
                let v2 = self.mesh.vertices[i2 as usize];

                let normal = (v1.position - v0.position)
                    .cross(v2.position - v0.position)
                    .normalize();

                let pos =
                    transform.transform_point3((v0.position + v1.position + v2.position) / 3.0);

                node.lines.push(DebugLine {
                    from: pos,
                    to: pos + transform.transform_vector3(normal) * 0.2,
                    width: self.width,
                    color,
                    depth: self.depth,
                });
            }
        }

        if let Some(color) = self.vertex_normals {
            for vertex in self.mesh.vertices.iter() {
                let pos = transform.transform_point3(vertex.position);

                node.lines.push(DebugLine {
                    from: pos,
                    to: pos + transform.transform_vector3(vertex.normal) * 0.2,
                    width: self.width,
                    color,
                    depth: self.depth,
                })
            }
        }
    }
}
*/

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct RawDebugLine {
    pub transform: Mat4,
    pub color: Color,
    pub depth: Vec2,
    pub padding: [u8; 8],
}

#[derive(Default)]
pub struct DebugNode {
    pub(crate) lines: Vec<DebugLine>,
    vertex_buffer: Option<Buffer>,
    pipelines: HashMap<ike_wgpu::TextureFormat, ike_wgpu::RenderPipeline>,
}

impl DebugNode {
    pub const HDR_TEXTURE: &'static str = "hdr_texture";
    pub const DEPTH_TEXTURE: &'static str = "depth_texture";
    pub const CAMERA: &'static str = "camera";
}

impl RenderNode for DebugNode {
    #[inline]
    fn input(&self) -> Vec<EdgeSlotInfo> {
        vec![
            EdgeSlotInfo::new::<RenderTexture>(Self::HDR_TEXTURE),
            EdgeSlotInfo::new::<Camera>(Self::CAMERA),
        ]
    }

    #[inline]
    fn run(
        &mut self,
        encoder: &mut ike_wgpu::CommandEncoder,
        input: &NodeEdge,
        output: &mut NodeEdge,
    ) -> Result<(), GraphError> {
        let target = input.get::<RenderTexture>(Self::HDR_TEXTURE)?;
        let camera = input.get::<Camera>(Self::CAMERA)?;

        let pipeline = self
            .pipelines
            .entry(target.format)
            .or_insert_with(|| create_pipeline(target.format, target.samples));

        let view_proj = camera.view_proj();
        let aspect = target.size.x as f32 / target.size.y as f32;

        let data = self
            .lines
            .iter()
            .map(|line| {
                let from = view_proj * line.from.extend(1.0);
                let to = view_proj * line.to.extend(1.0);

                let mut from = from.xyz() / from.w;
                let mut to = to.xyz() / to.w;

                from.x *= aspect;
                to.x *= aspect;

                let depth = if line.depth {
                    Vec2::new((from.z + to.z) / 2.0, (to.z - from.z) / 2.0)
                } else {
                    Vec2::new(0.0, 0.0)
                };

                from.z = 0.0;
                to.z = 0.0;

                let mut transform = Transform::from_translation((from + to) / 2.0);
                transform.look_at(transform.translation + Vec3::Z, to - transform.translation);
                transform.scale.x = line.width;
                transform.scale.y = from.distance(to) / 2.0;

                let transform = Transform::from_scale(Vec3::new(1.0 / aspect, 1.0, 1.0)).matrix()
                    * transform.matrix();

                RawDebugLine {
                    transform,
                    color: line.color,
                    depth,
                    padding: Default::default(),
                }
            })
            .collect::<Vec<_>>();

        let buffer = if let Some(ref mut buffer) = self.vertex_buffer {
            buffer.write(cast_slice(&data));

            buffer
        } else {
            let mut vertex_buffer =
                Buffer::new(ike_wgpu::BufferUsages::COPY_DST | ike_wgpu::BufferUsages::VERTEX);

            vertex_buffer.write(cast_slice(&data));

            self.vertex_buffer = Some(vertex_buffer);

            self.vertex_buffer.as_mut().unwrap()
        };

        let mut render_pass = encoder.begin_render_pass(&ike_wgpu::RenderPassDescriptor {
            label: Some("debug pass"),
            color_attachments: &[],
            depth_stencil_attachment: None,
        });

        render_pass.set_pipeline(pipeline);

        render_pass.set_vertex_buffer(0, buffer.raw().slice(..));

        render_pass.draw(0..6, 0..self.lines.len() as u32);

        Ok(())
    }
}
