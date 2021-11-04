use std::collections::HashMap;

use bytemuck::cast_slice;
use glam::{Vec3, Vec4Swizzles};
use ike_core::World;
use ike_render::{wgpu::ColorTargetState, *};
use ike_transform::Transform;

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
                    array_stride: 88,
                    step_mode: wgpu::VertexStepMode::Instance,
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
                        wgpu::VertexAttribute {
                            offset: 32,
                            format: wgpu::VertexFormat::Float32x4,
                            shader_location: 2,
                        },
                        wgpu::VertexAttribute {
                            offset: 48,
                            format: wgpu::VertexFormat::Float32x4,
                            shader_location: 3,
                        },
                        wgpu::VertexAttribute {
                            offset: 64,
                            format: wgpu::VertexFormat::Float32x4,
                            shader_location: 4,
                        },
                        wgpu::VertexAttribute {
                            offset: 80,
                            format: wgpu::VertexFormat::Float32x2,
                            shader_location: 5,
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
            primitive: Default::default(),
            multisample: wgpu::MultisampleState {
                count: target.samples,
                ..Default::default()
            },
            depth_stencil: None,
        });

        self.pipelines.insert(target, pipeline);
    }
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Instance {
    transform: [[f32; 4]; 4],
    color: [f32; 4],
    depth: [f32; 2],
}

pub struct DebugLineNode {
    instance_buffer: Buffer,
}

impl DebugLineNode {
    pub const TARGET: &'static str = "target";
    pub const CAMERA: &'static str = "camera";

    pub fn new() -> Self {
        Self {
            instance_buffer: Buffer::new(wgpu::BufferUsages::VERTEX),
        }
    }
}

impl RenderNode for DebugLineNode {
    fn input(&self) -> Vec<EdgeSlotInfo> {
        vec![
            EdgeSlotInfo::new::<RenderTexture>(Self::TARGET),
            EdgeSlotInfo::new::<Camera>(Self::CAMERA),
        ]
    }

    fn update(&mut self, world: &mut World) {
        world.init_resource::<ShaderResources>();
    }

    fn run(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        world: &World,
        input: &NodeInput,
        _output: &mut NodeEdge,
    ) -> Result<(), GraphError> {
        let target = input.get::<RenderTexture>(Self::TARGET)?;

        let aspect = target.size.x as f32 / target.size.y as f32;

        let camera = input.get::<Camera>(Self::CAMERA)?;
        let view_proj = camera.view_proj();

        let mut instances = Vec::new();

        while let Some(debug_line) = DEBUG_LINES.pop() {
            let from = view_proj * debug_line.from.extend(1.0);
            let to = view_proj * debug_line.to.extend(1.0);
            let mut from = from.xyz() / from.w;
            let mut to = to.xyz() / to.w;

            from.x *= aspect;
            to.x *= aspect;

            let depth = if debug_line.use_depth {
                [(from.z + to.z) / 2.0, (to.z - from.z) / 2.0]
            } else {
                [0.0; 2]
            };

            let mut transform = Transform::from_translation((from + to) / 2.0);
            transform.look_at(transform.translation + Vec3::Z, to - transform.translation);
            transform.scale.x = debug_line.width;
            transform.scale.y = from.distance(to) / 2.0;

            let transform = Transform::from_scale(Vec3::new(1.0 / aspect, 1.0, 1.0)).matrix()
                * transform.matrix();

            instances.push(Instance {
                transform: transform.to_cols_array_2d(),
                color: [1.0; 4],
                depth,
            });
        }

        self.instance_buffer.write(cast_slice(&instances));

        let texture = target.texture();

        let view = texture.create_view(&Default::default());

        let mut resources = world.write_resource::<ShaderResources>().unwrap();

        if !resources.pipelines.contains_key(&target.target()) {
            resources.create_pipeline(target.target());
        }

        let pipeline = &resources.pipelines[&target.target()];

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
            depth_stencil_attachment: None,
        });

        render_pass.set_pipeline(pipeline);

        render_pass.set_vertex_buffer(0, self.instance_buffer.raw().slice(..));

        render_pass.draw(0..6, 0..instances.len() as u32);

        Ok(())
    }
}
