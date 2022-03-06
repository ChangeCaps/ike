use std::mem;

use bytemuck::{bytes_of, Pod, Zeroable};
use ike_assets::{Assets, Handle};
use ike_ecs::{FromResources, Resources, World};
use ike_math::Mat4;
use ike_render::{
    include_wgsl, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
    BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, Buffer, BufferBindingType,
    BufferDescriptor, BufferUsages, ColorTargetState, ColorWrites, FragmentState, IndexFormat,
    LoadOp, Mesh, MeshBuffers, Operations, PipelineLayout, PipelineLayoutDescriptor, RawCamera,
    RawColor, RenderContext, RenderDevice, RenderGraphContext, RenderNode,
    RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor,
    RenderQueue, ShaderStages, SlotInfo, TextureFormat, TextureView, VertexAttribute,
    VertexBufferLayout, VertexFormat, VertexState, VertexStepMode,
};
use ike_transform::GlobalTransform;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Object {
    transform: [[f32; 4]; 4],
    view_proj: [[f32; 4]; 4],
    camera_position: [f32; 3],
    _padding: [u8; 4],
}

impl Object {
    pub fn new(transform: Mat4, camera: &RawCamera) -> Self {
        Self {
            transform: transform.to_cols_array_2d(),
            view_proj: camera.view_proj().to_cols_array_2d(),
            camera_position: camera.position.into(),
            _padding: [0u8; 4],
        }
    }
}

struct ObjectBinding {
    buffer: Buffer,
    bind_group: BindGroup,
}

impl ObjectBinding {
    pub fn new(device: &RenderDevice, layout: &BindGroupLayout) -> Self {
        let buffer = device.create_buffer(&BufferDescriptor {
            label: Some("ike_pbr_oject_binding_buffer"),
            size: mem::size_of::<Object>() as u64,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("ike_pbr_oject_binding_group"),
            layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: buffer.raw().as_entire_binding(),
            }],
        });

        Self { buffer, bind_group }
    }

    pub fn write(&self, queue: &RenderQueue, object: &Object) {
        queue.write_buffer(&self.buffer, 0, bytes_of(object));
    }
}

pub struct PbrResources {
    pub object_bind_group_layout: BindGroupLayout,
    pub pipeline_layout: PipelineLayout,
    pub render_pipeline: RenderPipeline,
}

impl FromResources for PbrResources {
    fn from_resources(resources: &Resources) -> Self {
        let device = resources.read::<RenderDevice>().unwrap();

        let object_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("ike_pbr_oject_binding_group_layout"),
                entries: &[BindGroupLayoutEntry {
                    binding: 0,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    visibility: ShaderStages::VERTEX_FRAGMENT,
                    count: None,
                }],
            });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("ike_pbr_pipeline_layout"),
            bind_group_layouts: &[&object_bind_group_layout],
            push_constant_ranges: &[],
        });

        let module = &device.create_shader_module(&include_wgsl!("pbr.wgsl"));

        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("ike_pbr_render_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module,
                entry_point: "vert",
                buffers: &[
                    VertexBufferLayout {
                        step_mode: VertexStepMode::Vertex,
                        array_stride: 12,
                        attributes: &[VertexAttribute {
                            format: VertexFormat::Float32x3,
                            shader_location: 0,
                            offset: 0,
                        }],
                    },
                    VertexBufferLayout {
                        step_mode: VertexStepMode::Vertex,
                        array_stride: 12,
                        attributes: &[VertexAttribute {
                            format: VertexFormat::Float32x3,
                            shader_location: 1,
                            offset: 0,
                        }],
                    },
                ],
            },
            fragment: Some(FragmentState {
                module,
                entry_point: "frag",
                targets: &[ColorTargetState {
                    format: TextureFormat::Bgra8UnormSrgb,
                    blend: None,
                    write_mask: ColorWrites::ALL,
                }],
            }),
            depth_stencil: None,
            multisample: Default::default(),
            primitive: Default::default(),
            multiview: None,
        });

        Self {
            object_bind_group_layout,
            pipeline_layout,
            render_pipeline,
        }
    }
}

#[derive(Default)]
pub struct PbrNode {
    object_bindings: Vec<ObjectBinding>,
}

impl PbrNode {
    pub const RENDER_TARGET: &'static str = "render_target";
    pub const DEPTH: &'static str = "depth";
    pub const CAMERA: &'static str = "camera";
}

impl RenderNode for PbrNode {
    fn input() -> Vec<SlotInfo> {
        vec![
            SlotInfo::new::<TextureView>(Self::RENDER_TARGET),
            SlotInfo::new::<RawCamera>(Self::CAMERA),
        ]
    }

    fn update(&mut self, world: &mut World) {
        let render_device = world.resource::<RenderDevice>();
        let meshes = world.resource::<Assets<Mesh>>();
        let mut mesh_buffers = world.resource_mut::<Assets<MeshBuffers>>();

        for mesh_handle in world.query::<&Handle<Mesh>>().unwrap().iter() {
            let mesh_buffers_id = mesh_handle.cast::<MeshBuffers>();

            if !mesh_buffers.contains(&mesh_buffers_id) {
                let mesh = meshes.get(mesh_handle).unwrap();
                mesh_buffers.insert(
                    mesh_buffers_id,
                    MeshBuffers::from_mesh(mesh, &render_device),
                );
            }
        }
    }

    fn run(
        &mut self,
        graph_context: &mut RenderGraphContext<'_>,
        render_context: &mut RenderContext,
        world: &World,
    ) -> ike_render::RenderGraphResult<()> {
        let target = graph_context.get_input::<TextureView>(Self::RENDER_TARGET)?;
        let camera = graph_context.get_input::<RawCamera>(Self::CAMERA)?;

        let meshes = world.resource::<Assets<Mesh>>();
        let mesh_buffers = world.resource::<Assets<MeshBuffers>>();
        let pbr_resources = world.resource::<PbrResources>();

        let mesh_query = world
            .query::<(&Handle<Mesh>, Option<&GlobalTransform>)>()
            .unwrap();

        for (i, (_, global_transform)) in mesh_query.iter().enumerate() {
            if self.object_bindings.len() <= i {
                self.object_bindings.push(ObjectBinding::new(
                    &render_context.device,
                    &pbr_resources.object_bind_group_layout,
                ));
            }

            let transform = global_transform.map_or(Mat4::IDENTITY, |transform| transform.matrix());

            self.object_bindings[i].write(&render_context.queue, &Object::new(transform, camera));
        }

        let mut render_pass = render_context
            .encoder
            .begin_render_pass(&RenderPassDescriptor {
                label: Some("pbr_pass"),
                color_attachments: &[RenderPassColorAttachment {
                    view: target.raw(),
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(RawColor::BLACK),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });

        render_pass.set_pipeline(&pbr_resources.render_pipeline);

        for (i, (mesh_handle, _)) in mesh_query.iter().enumerate() {
            let object_binding = &self.object_bindings[i];
            let mesh = meshes.get(mesh_handle).unwrap();
            let mesh_buffers = mesh_buffers.get(mesh_handle.cast::<MeshBuffers>()).unwrap();

            if let Some(position) = mesh_buffers.get_attribute(Mesh::POSITION) {
                render_pass.set_vertex_buffer(0, position.raw().slice(..));
            } else {
                continue;
            }

            if let Some(normal) = mesh_buffers.get_attribute(Mesh::NORMAL) {
                render_pass.set_vertex_buffer(1, normal.raw().slice(..));
            } else {
                continue;
            }

            render_pass.set_index_buffer(
                mesh_buffers.get_indices().raw().slice(..),
                IndexFormat::Uint32,
            );

            render_pass.set_bind_group(0, &object_binding.bind_group, &[]);

            render_pass.draw_indexed(0..mesh.get_indices().len() as u32, 0, 0..1);
        }

        Ok(())
    }
}
