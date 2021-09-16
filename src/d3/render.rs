use std::collections::{BTreeMap, HashMap};

use bytemuck::bytes_of;
use glam::{Mat4, Vec3};

use crate::{
    id::{HasId, Id},
    prelude::{Camera, Color},
    renderer::{Drawable, PassNode, PassNodeCtx, RenderCtx, SampleCount, TargetFormat},
};

use super::{
    default_pipeline::default_pipeline, BufferVersion, Indices, Mesh, Transform3d, Vertices,
};

pub(crate) struct SizedBuffer {
    len: usize,
    pub(crate) buffer: ike_wgpu::Buffer,
    usage: ike_wgpu::BufferUsages,
}

impl SizedBuffer {
    pub fn new(device: &ike_wgpu::Device, data: &[u8], mut usage: ike_wgpu::BufferUsages) -> Self {
        usage |= ike_wgpu::BufferUsages::COPY_DST;

        let buffer = device.create_buffer_init(&ike_wgpu::BufferInitDescriptor {
            label: None,
            contents: data,
            usage,
        });

        Self {
            len: data.len(),
            buffer,
            usage,
        }
    }

    pub fn write(&mut self, device: &ike_wgpu::Device, queue: &ike_wgpu::Queue, data: &[u8]) {
        if self.len < data.len() {
            let buffer = device.create_buffer_init(&ike_wgpu::BufferInitDescriptor {
                label: None,
                contents: data,
                usage: self.usage,
            });

            self.buffer = buffer;
            self.len = data.len();
        } else {
            queue.write_buffer(&self.buffer, 0, data);
        }
    }
}

struct VersionedBuffer {
    version: BufferVersion,
    buffer: SizedBuffer,
}

#[derive(PartialEq, Eq, Hash, PartialOrd, Ord)]
struct InstancesId {
    vertex_buffer: Id<Vertices>,
    index_buffer: Id<Indices>,
}

#[derive(Default)]
struct Instances {
    count: u32,
    data: Vec<u8>,
    buffer: Option<SizedBuffer>,
}

#[derive(Default)]
pub(crate) struct Meshes {
    vertex_buffers: BTreeMap<Id<Vertices>, VersionedBuffer>,
    index_buffers: BTreeMap<Id<Indices>, VersionedBuffer>,
    instances: BTreeMap<InstancesId, Instances>,
}

impl Meshes {
    #[inline]
    pub fn add_instance<V: bytemuck::Pod>(&mut self, ctx: &RenderCtx, mesh: &Mesh<V>, data: &[u8]) {
        let mesh_data = mesh.data();

        if let Some(buffer) = self.vertex_buffers.get_mut(&mesh.vertices.id()) {
            if mesh.vertices.changed(buffer.version) {
                buffer
                    .buffer
                    .write(&ctx.device, &ctx.queue, &mesh_data.vertex_data);

                buffer.version = mesh.vertices.version();
            }
        } else {
            self.vertex_buffers.insert(
                mesh.vertices.id(),
                VersionedBuffer {
                    version: Default::default(),
                    buffer: SizedBuffer::new(
                        &ctx.device,
                        &mesh_data.vertex_data,
                        ike_wgpu::BufferUsages::VERTEX,
                    ),
                },
            );
        }

        if let Some(buffer) = self.index_buffers.get_mut(&mesh.indices.id()) {
            if mesh.indices.changed(buffer.version) {
                buffer
                    .buffer
                    .write(&ctx.device, &ctx.queue, &mesh_data.index_data);

                buffer.version = mesh.vertices.version();
            }
        } else {
            self.index_buffers.insert(
                mesh.indices.id(),
                VersionedBuffer {
                    version: Default::default(),
                    buffer: SizedBuffer::new(
                        &ctx.device,
                        &mesh_data.index_data,
                        ike_wgpu::BufferUsages::INDEX,
                    ),
                },
            );
        }

        let id = InstancesId {
            vertex_buffer: mesh.vertices.id(),
            index_buffer: mesh.indices.id(),
        };

        let instances = self.instances.entry(id).or_default();

        let len = instances.data.len();

        instances.count += 1;

        instances.data.resize(len + data.len(), 0);
        instances.data[len..].copy_from_slice(data);
    }

    #[inline]
    pub fn prepare(&mut self, ctx: &RenderCtx) {
        for instances in self.instances.values_mut() {
            if let Some(ref mut buffer) = instances.buffer {
                buffer.write(&ctx.device, &ctx.queue, &instances.data);
            } else {
                instances.buffer = Some(SizedBuffer::new(
                    &ctx.device,
                    &instances.data,
                    ike_wgpu::BufferUsages::VERTEX,
                ));
            }
        }
    }

    #[inline]
    pub fn clear(&mut self) {
        for (_, instance) in &mut self.instances {
            instance.data.clear();
            instance.count = 0;
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PointLight {
    pub position: Vec3,
    pub intensity: f32,
    pub color: Color,
}

impl Drawable for PointLight {
    type Node = D3Node;

    #[inline]
    fn draw(&self, _ctx: &RenderCtx, node: &mut Self::Node) {
        node.point_lights.push(*self);
    }
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    pub view_proj: Mat4,
    pub point_light_count: u32,
    pub padding: [u8; 12],
    pub point_lights: [PointLight; 64],
}

#[inline]
fn point_lights(lights: &[PointLight]) -> [PointLight; 64] {
    let mut point_lights: [PointLight; 64] = bytemuck::Zeroable::zeroed();

    for (i, light) in lights.iter().enumerate() {
        point_lights[i] = *light;
    }

    point_lights
}

impl Drawable for Mesh {
    type Node = D3Node;

    #[inline]
    fn draw(&self, ctx: &RenderCtx, node: &mut Self::Node) {
        node.meshes
            .add_instance(ctx, self, bytes_of(&Mat4::IDENTITY));
    }
}

impl Drawable for (&Mesh, &Transform3d) {
    type Node = D3Node;

    #[inline]
    fn draw(&self, ctx: &RenderCtx, node: &mut Self::Node) {
        node.meshes
            .add_instance(ctx, self.0, bytes_of(&self.1.matrix()));
    }
}

impl Mesh {
    #[inline]
    pub fn transform<'a>(&'a self, transform: &'a Transform3d) -> (&Self, &Transform3d) {
        (self, transform)
    }
}

pub struct D3Node {
    pub(crate) meshes: Meshes,
    pub(crate) point_lights: Vec<PointLight>,
    uniforms_buffer: Option<ike_wgpu::Buffer>,
    uniforms_bind_group: Option<ike_wgpu::BindGroup>,
    default_pipelines: HashMap<ike_wgpu::TextureFormat, ike_wgpu::RenderPipeline>,
}

impl Default for D3Node {
    #[inline]
    fn default() -> Self {
        Self {
            meshes: Default::default(),
            point_lights: Vec::new(),
            uniforms_buffer: Default::default(),
            uniforms_bind_group: Default::default(),
            default_pipelines: Default::default(),
        }
    }
}

impl D3Node {
    #[inline]
    pub fn new() -> Self {
        Self {
            meshes: Default::default(),
            point_lights: Vec::new(),
            uniforms_buffer: Default::default(),
            uniforms_bind_group: Default::default(),
            default_pipelines: Default::default(),
        }
    }
}

impl<S> PassNode<S> for D3Node {
    #[inline]
    fn clear(&mut self) {
        self.meshes.clear();
        self.point_lights.clear();
    }

    #[inline]
    fn run<'a>(&'a mut self, ctx: &mut PassNodeCtx<'_, 'a>, _: &mut S) {
        let sample_count = ctx.data.get::<SampleCount>().unwrap_or(&SampleCount(1));
        let format = ctx
            .data
            .get::<TargetFormat>()
            .cloned()
            .unwrap_or_else(|| TargetFormat(ctx.view.format))
            .0;
        let camera = ctx.data.get::<Camera>().unwrap_or_else(|| &ctx.view.camera);

        let default_pipeline = self
            .default_pipelines
            .entry(format)
            .or_insert_with(|| default_pipeline(&ctx.render_ctx.device, format, sample_count.0));

        let uniforms = Uniforms {
            view_proj: camera.view_proj(),
            point_light_count: self.point_lights.len() as u32,
            padding: [0; 12],
            point_lights: point_lights(&self.point_lights),
        };

        if let Some(ref buffer) = self.uniforms_buffer {
            ctx.render_ctx
                .queue
                .write_buffer(buffer, 0, bytes_of(&uniforms));
        } else {
            let buffer =
                ctx.render_ctx
                    .device
                    .create_buffer_init(&ike_wgpu::BufferInitDescriptor {
                        label: None,
                        contents: bytes_of(&uniforms),
                        usage: ike_wgpu::BufferUsages::UNIFORM | ike_wgpu::BufferUsages::COPY_DST,
                    });

            self.uniforms_buffer = Some(buffer);
        }

        if self.uniforms_bind_group.is_none() {
            let bind_group_layout = ctx.render_ctx.device.create_bind_group_layout(
                &ike_wgpu::BindGroupLayoutDescriptor {
                    label: None,
                    entries: &[ike_wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        ty: ike_wgpu::BindingType::Buffer {
                            ty: ike_wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        visibility: ike_wgpu::ShaderStages::VERTEX_FRAGMENT,
                        count: None,
                    }],
                },
            );

            let bind_group =
                ctx.render_ctx
                    .device
                    .create_bind_group(&ike_wgpu::BindGroupDescriptor {
                        label: None,
                        layout: &bind_group_layout,
                        entries: &[ike_wgpu::BindGroupEntry {
                            binding: 0,
                            resource: self.uniforms_buffer.as_ref().unwrap().as_entire_binding(),
                        }],
                    });

            self.uniforms_bind_group = Some(bind_group);
        }

        self.meshes.prepare(&ctx.render_ctx);

        ctx.render_pass.set_pipeline(default_pipeline);

        ctx.render_pass
            .set_bind_group(0, self.uniforms_bind_group.as_ref().unwrap(), &[]);

        for (id, instances) in &mut self.meshes.instances {
            let vertex_buffer = self.meshes.vertex_buffers.get(&id.vertex_buffer).unwrap();
            let index_buffer = self.meshes.index_buffers.get(&id.index_buffer).unwrap();

            ctx.render_pass.set_vertex_buffer(
                0,
                vertex_buffer
                    .buffer
                    .buffer
                    .slice(..vertex_buffer.buffer.len as u64),
            );

            let instance_buffer = instances.buffer.as_ref().unwrap();

            ctx.render_pass.set_vertex_buffer(
                1,
                instance_buffer.buffer.slice(..instance_buffer.len as u64),
            );

            ctx.render_pass.set_index_buffer(
                index_buffer
                    .buffer
                    .buffer
                    .slice(..index_buffer.buffer.len as u64),
                ike_wgpu::IndexFormat::Uint32,
            );

            ctx.render_pass.draw_indexed(
                0..index_buffer.buffer.len as u32 / 4,
                0,
                0..instances.count,
            );
        }
    }
}
