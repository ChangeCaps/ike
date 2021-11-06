use std::collections::HashMap;

use bytemuck::cast_slice;
use glam::{Vec3Swizzles, Vec4, Vec4Swizzles};
use ike_core::WorldRef;
use ike_render::{wgpu::ColorTargetState, *};

use crate::DEBUG_LINES;

struct ShaderResources {
    shader: wgpu::ShaderModule,
    pipelines: HashMap<RenderTarget, wgpu::RenderPipeline>,
}

impl Default for ShaderResources {
    fn default() -> Self {
        Self::new()
    }
}

impl ShaderResources {
    pub fn new() -> Self {
        let device = render_device();

        let shader = device.create_shader_module(&wgpu::include_wgsl!("shader.wgsl"));

        Self {
            shader,
            pipelines: HashMap::new(),
        }
    }

    fn create_pipeline(&mut self, target: RenderTarget) {
        let device = render_device();

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("debug line pipeline"),
            layout: None,
            vertex: wgpu::VertexState {
                module: &self.shader,
                entry_point: "main",
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: 32,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[
                        wgpu::VertexAttribute {
                            offset: 0,
                            format: wgpu::VertexFormat::Float32x4,
                            shader_location: 0,
                        },
                        wgpu::VertexAttribute {
                            offset: 16,
                            format: wgpu::VertexFormat::Float32x4,
                            shader_location: 1,
                        },
                    ],
                }],
            },
            fragment: Some(wgpu::FragmentState {
                module: &self.shader,
                entry_point: "main",
                targets: &[ColorTargetState {
                    format: target.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                }],
            }),
            primitive: wgpu::PrimitiveState {
                cull_mode: Some(wgpu::Face::Back),
                ..Default::default()
            },
            multisample: wgpu::MultisampleState {
                count: target.samples,
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_compare: wgpu::CompareFunction::LessEqual,
                depth_write_enabled: true,
                stencil: Default::default(),
                bias: Default::default(),
            }),
        });

        self.pipelines.insert(target, pipeline);
    }
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 4],
    color: [f32; 4],
}

impl Vertex {
    #[inline]
    pub fn new(position: Vec4, color: Color) -> Self {
        Self {
            position: position.into(),
            color: color.into(),
        }
    }
}

pub struct DebugLineNode {
    buffer: Buffer,
}

impl DebugLineNode {
    pub const TARGET: &'static str = "target";
    pub const DEPTH: &'static str = "depth";
    pub const CAMERA: &'static str = "camera";

    pub fn new() -> Self {
        Self {
            buffer: Buffer::new(wgpu::BufferUsages::VERTEX),
        }
    }
}

impl RenderNode for DebugLineNode {
    fn input(&self) -> Vec<EdgeSlotInfo> {
        vec![
            EdgeSlotInfo::new::<RenderTexture>(Self::TARGET),
            EdgeSlotInfo::new::<wgpu::TextureView>(Self::DEPTH),
            EdgeSlotInfo::new::<Camera>(Self::CAMERA),
        ]
    }

    fn update(&mut self, world: &WorldRef) {
        world.init_resource::<ShaderResources>();
    }

    fn run(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        world: &WorldRef,
        input: &NodeInput,
        _output: &mut NodeEdge,
    ) -> Result<(), GraphError> {
        let target = input.get::<RenderTexture>(Self::TARGET)?;
        let depth = input.get::<wgpu::TextureView>(Self::DEPTH)?;

        let camera = input.get::<Camera>(Self::CAMERA)?;
        let view_proj = camera.view_proj();

        let mut vertices: Vec<Vertex> = Vec::new();

        while let Some(debug_line) = DEBUG_LINES.pop() {
            let from = view_proj * debug_line.from.extend(1.0);
            let to = view_proj * debug_line.to.extend(1.0);
            let fw = from.w;
            let tw = to.w;
            let from = from.xyz() / from.w;
            let to = to.xyz() / to.w;

            let f = from.xy();
            let t = to.xy();

            let dir = t - f;
            let p = dir.normalize().perp();

            let a = f + p * debug_line.width;
            let b = f - p * debug_line.width;
            let c = t + p * debug_line.width;
            let d = t - p * debug_line.width;

            let (df, dt) = if debug_line.use_depth {
                (from.z, to.z)
            } else {
                (0.0, 0.0)
            };

            let a = Vec4::new(a.x * fw, a.y * fw, df * fw, fw);
            let b = Vec4::new(b.x * fw, b.y * fw, df * fw, fw);
            let c = Vec4::new(c.x * tw, c.y * tw, dt * tw, tw);
            let d = Vec4::new(d.x * tw, d.y * tw, dt * tw, tw);

            vertices.push(Vertex::new(a, debug_line.color));
            vertices.push(Vertex::new(b, debug_line.color));
            vertices.push(Vertex::new(c, debug_line.color));
            vertices.push(Vertex::new(c, debug_line.color));
            vertices.push(Vertex::new(b, debug_line.color));
            vertices.push(Vertex::new(d, debug_line.color));
        }

        self.buffer.write(cast_slice(&vertices));

        let texture = target.texture();

        let view = texture.create_view(&Default::default());

        let mut resources = world.get_resource_mut::<ShaderResources>().unwrap();

        if !resources.pipelines.contains_key(&target.target()) {
            resources.create_pipeline(target.target());
        }

        let pipeline = &resources.pipelines[&target.target()];

        if vertices.is_empty() {
            return Ok(());
        }

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: true,
                },
            }],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: depth,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: true,
                }),
                stencil_ops: None,
            }),
        });

        render_pass.set_pipeline(pipeline);

        render_pass.set_vertex_buffer(0, self.buffer.raw().slice(..));

        render_pass.draw(0..vertices.len() as u32, 0..1);

        Ok(())
    }
}
