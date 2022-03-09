use bytemuck::{cast_slice, Pod, Zeroable};
use crossbeam::queue::SegQueue;
use ike_ecs::{FromWorld, World};
use ike_math::Vec3;
use ike_render::{
    include_wgsl, Buffer, BufferDescriptor, BufferUsages, Color, ColorTargetState, ColorWrites,
    CompareFunction, DepthStencilState, FragmentState, LoadOp, Operations, PrimitiveState,
    PrimitiveTopology, RawCamera, RenderContext, RenderDevice, RenderGraphContext,
    RenderGraphResult, RenderNode, RenderPassColorAttachment, RenderPassDepthStencilAttachemnt,
    RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor, SlotInfo, Surface,
    TextureFormat, TextureView, VertexAttribute, VertexBufferLayout, VertexFormat, VertexState,
    VertexStepMode,
};

pub static DEBUG_LINES: SegQueue<DebugLine> = SegQueue::new();

#[derive(Clone, Copy, Debug)]
pub struct DebugLine {
    pub from: Vec3,
    pub to: Vec3,
    pub color: Color,
    pub always_visible: bool,
    pub project: bool,
}

impl DebugLine {
    pub const fn new(to: Vec3, from: Vec3) -> Self {
        Self {
            to,
            from,
            color: Color::GREEN,
            always_visible: false,
            project: true,
        }
    }

    pub const fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub const fn always_visible(mut self) -> Self {
        self.always_visible = true;
        self
    }

    pub const fn without_projection(mut self) -> Self {
        self.project = false;
        self
    }

    pub fn draw(self) {
        DEBUG_LINES.push(self);
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct DebugLineVertex {
    pub position: [f32; 4],
    pub color: [f32; 4],
}

pub struct DebugLinePipeline {
    pub render_pipeline: RenderPipeline,
}

impl FromWorld for DebugLinePipeline {
    fn from_world(world: &mut World) -> Self {
        let device = world.resource::<RenderDevice>();
        let surface = world.resource::<Surface>();

        let module = &device.create_shader_module(&include_wgsl!("debug_line.wgsl"));

        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("ike_debug_line_render_pipeline"),
            layout: None,
            vertex: VertexState {
                module,
                entry_point: "vert",
                buffers: &[VertexBufferLayout {
                    array_stride: 32,
                    step_mode: VertexStepMode::Vertex,
                    attributes: &[
                        VertexAttribute {
                            format: VertexFormat::Float32x4,
                            shader_location: 0,
                            offset: 0,
                        },
                        VertexAttribute {
                            format: VertexFormat::Float32x4,
                            shader_location: 1,
                            offset: 16,
                        },
                    ],
                }],
            },
            fragment: Some(FragmentState {
                module,
                entry_point: "frag",
                targets: &[ColorTargetState {
                    format: surface.format(),
                    blend: None,
                    write_mask: ColorWrites::ALL,
                }],
            }),
            depth_stencil: Some(DepthStencilState {
                format: TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: CompareFunction::GreaterEqual,
                stencil: Default::default(),
                bias: Default::default(),
            }),
            multisample: Default::default(),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::LineList,
                ..Default::default()
            },
            multiview: None,
        });

        Self { render_pipeline }
    }
}

#[derive(Default)]
pub struct DebugLineNode {
    buffer_len: usize,
    buffer: Option<Buffer>,
}

impl DebugLineNode {
    pub const CAMERA: &'static str = "camera";
    pub const TARGET: &'static str = "target";
    pub const DEPTH: &'static str = "depth";
}

impl RenderNode for DebugLineNode {
    fn input() -> Vec<SlotInfo> {
        vec![
            SlotInfo::new::<RawCamera>(Self::CAMERA),
            SlotInfo::new::<TextureView>(Self::TARGET),
            SlotInfo::new::<TextureView>(Self::DEPTH),
        ]
    }

    fn run(
        &mut self,
        graph_context: &mut RenderGraphContext<'_>,
        render_context: &mut RenderContext,
        world: &World,
    ) -> RenderGraphResult<()> {
        let camera = graph_context.get_input::<RawCamera>(Self::CAMERA)?;
        let target = graph_context.get_input::<TextureView>(Self::TARGET)?;
        let depth = graph_context.get_input::<TextureView>(Self::DEPTH)?;

        let pipeline = world.resource::<DebugLinePipeline>();

        if DEBUG_LINES.is_empty() {
            return Ok(());
        }

        let mut vertices: Vec<DebugLineVertex> = Vec::with_capacity(DEBUG_LINES.len() * 2);

        while let Some(debug_line) = DEBUG_LINES.pop() {
            let mut from = debug_line.from.extend(1.0);
            let mut to = debug_line.to.extend(1.0);

            if debug_line.project {
                from = camera.project_point(from);
                to = camera.project_point(to);
            }

            if debug_line.always_visible {
                from.z = 0.0;
                to.z = 0.0;
            }

            vertices.push(DebugLineVertex {
                position: from.into(),
                color: debug_line.color.into(),
            });

            vertices.push(DebugLineVertex {
                position: to.into(),
                color: debug_line.color.into(),
            });
        }

        if self.buffer.is_none() || self.buffer_len < vertices.len() {
            self.buffer_len = vertices.len();

            let buffer = render_context.device.create_buffer(&BufferDescriptor {
                label: Some("debug_line_vertex_buffer"),
                size: 32 * vertices.len() as u64,
                usage: BufferUsages::COPY_DST | BufferUsages::VERTEX,
                mapped_at_creation: false,
            });

            self.buffer = Some(buffer);
        }

        let buffer = self.buffer.as_ref().unwrap();

        render_context
            .queue
            .write_buffer(buffer, 0, cast_slice(&vertices));

        let mut render_pass = render_context
            .encoder
            .begin_render_pass(&RenderPassDescriptor {
                label: Some("ike_debug_line_pass"),
                color_attachments: &[RenderPassColorAttachment {
                    view: target.raw(),
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Load,
                        store: true,
                    },
                }],
                depth_stencil_attachment: Some(RenderPassDepthStencilAttachemnt {
                    view: depth.raw(),
                    stencil_ops: None,
                    depth_ops: Some(Operations {
                        load: LoadOp::Load,
                        store: true,
                    }),
                }),
            });

        render_pass.set_pipeline(&pipeline.render_pipeline);

        render_pass.set_vertex_buffer(0, buffer.raw().slice(..));

        render_pass.draw(0..vertices.len() as u32, 0..1);

        Ok(())
    }
}
