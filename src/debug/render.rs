use std::collections::HashMap;

use bytemuck::cast_slice;
use glam::Vec3;

use crate::{
    d3::SizedBuffer,
    prelude::{Camera, Transform3d},
    renderer::{Drawable, PassNode, PassNodeCtx, RenderCtx, SampleCount, TargetFormat},
};

fn create_pipeline(
    device: &ike_wgpu::Device,
    format: ike_wgpu::TextureFormat,
    sample_count: u32,
) -> ike_wgpu::RenderPipeline {
    let module = device.create_shader_module(&ike_wgpu::include_wgsl!("shader.wgsl"));

    device.create_render_pipeline(&ike_wgpu::RenderPipelineDescriptor {
        label: None,
        layout: None,
        vertex: ike_wgpu::VertexState {
            module: &module,
            entry_point: "main",
            buffers: &[ike_wgpu::VertexBufferLayout {
                array_stride: 64,
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
}

impl DebugLine {
    #[inline]
    pub fn new(from: Vec3, to: Vec3) -> Self {
        Self {
            from,
            to,
            width: 0.002,
        }
    }
}

impl Drawable for DebugLine {
    type Node = DebugNode;

    #[inline]
    fn draw(&self, _ctx: &RenderCtx, node: &mut Self::Node) {
        node.lines.push(self.clone());
    }
}

#[derive(Default)]
pub struct DebugNode {
    pub(crate) lines: Vec<DebugLine>,
    vertex_buffer: Option<SizedBuffer>,
    pipelines: HashMap<ike_wgpu::TextureFormat, ike_wgpu::RenderPipeline>,
}

impl<S> PassNode<S> for DebugNode {
    #[inline]
    fn clear(&mut self) {
        self.lines.clear();
    }

    #[inline]
    fn run<'a>(&'a mut self, ctx: &mut PassNodeCtx<'_, 'a>, _: &mut S) {
        let format = ctx
            .data
            .get::<TargetFormat>()
            .map_or_else(|| ctx.view.format, |f| f.0);
        let sample_count = ctx.data.get::<SampleCount>().map_or(0, |f| f.0);
        let camera = ctx.data.get::<Camera>().unwrap_or_else(|| &ctx.view.camera);

        let pipeline = self
            .pipelines
            .entry(format)
            .or_insert_with(|| create_pipeline(&ctx.render_ctx.device, format, sample_count));

        let view_proj = camera.view_proj();

        let data = self
            .lines
            .iter()
            .map(|line| {
                let mut from = view_proj.transform_point3(line.from);
                let mut to = view_proj.transform_point3(line.to);

                from.z = 0.0;
                to.z = 0.0;

                let mut transform = Transform3d::from_translation((from + to) / 2.0);
                transform.look_at(transform.translation + Vec3::Z, to);
                transform.scale.x = line.width;
                transform.scale.y = from.distance(to) / 2.0;

                transform.matrix()
            })
            .collect::<Vec<_>>();

        let buffer = if let Some(ref mut buffer) = self.vertex_buffer {
            buffer.write(
                &ctx.render_ctx.device,
                &ctx.render_ctx.queue,
                cast_slice(&data),
            );

            buffer
        } else {
            let vertex_buffer = SizedBuffer::new(
                &ctx.render_ctx.device,
                cast_slice(&data),
                ike_wgpu::BufferUsages::COPY_DST | ike_wgpu::BufferUsages::VERTEX,
            );

            self.vertex_buffer = Some(vertex_buffer);

            self.vertex_buffer.as_ref().unwrap()
        };

        ctx.render_pass.set_pipeline(pipeline);

        ctx.render_pass
            .set_vertex_buffer(0, buffer.buffer.slice(..));

        ctx.render_pass.draw(0..6, 0..self.lines.len() as u32);
    }
}
