use std::collections::HashMap;

use bytemuck::bytes_of;

use crate::{
    id::{HasId, Id},
    renderer::{PassNode, PassNodeCtx, RenderCtx, SampleCount, TargetFormat, TargetSize, ViewProj},
};

use super::{
    default_pipeline::default_pipeline, BufferVersion, Indices, Mesh, Transform3d, Vertices,
};

pub struct Render3dCtx<'a> {
    pub meshes: &'a mut Meshes,
    pub render_ctx: &'a RenderCtx,
}

pub trait Render3d: Sized {
    fn render(&mut self, ctx: &mut Render3dCtx);

    fn default_pipeline(
        &mut self,
        device: &ike_wgpu::Device,
        format: ike_wgpu::TextureFormat,
        sample_count: u32,
    ) -> ike_wgpu::RenderPipeline {
        default_pipeline::<Self>(self, device, format, sample_count)
    }
}

struct SizedBuffer {
    len: usize,
    buffer: ike_wgpu::Buffer,
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

#[derive(PartialEq, Eq, Hash)]
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
pub struct Meshes {
    vertex_buffers: HashMap<Id<Vertices>, VersionedBuffer>,
    index_buffers: HashMap<Id<Indices>, VersionedBuffer>,
    instances: HashMap<InstancesId, Instances>,
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

            instances.data.clear();
        }
    }
}

impl<'a> Render3dCtx<'a> {
    #[inline]
    pub fn draw_mesh<V: bytemuck::Pod>(&mut self, mesh: &Mesh<V>, transform: &Transform3d) {
        self.meshes
            .add_instance(self.render_ctx, mesh, bytes_of(&transform.matrix()));
    }
}

type RenderFn<S> = dyn Fn(&mut S, &mut Render3dCtx);
type PipelineFn<S> =
    dyn Fn(&mut S, &ike_wgpu::Device, ike_wgpu::TextureFormat, u32) -> ike_wgpu::RenderPipeline;

pub struct D3Node<S> {
    meshes: Meshes,
    camera_buffer: Option<ike_wgpu::Buffer>,
    camera_bind_group: Option<ike_wgpu::BindGroup>,
    default_pipelines: HashMap<ike_wgpu::TextureFormat, ike_wgpu::RenderPipeline>,
    render_fn: Box<RenderFn<S>>,
    default_pipeline_fn: Box<PipelineFn<S>>,
}

impl<S: Render3d + 'static> Default for D3Node<S> {
    #[inline]
    fn default() -> Self {
        Self {
            meshes: Default::default(),
            camera_buffer: Default::default(),
            camera_bind_group: Default::default(),
            default_pipelines: Default::default(),
            render_fn: Box::new(Render3d::render),
            default_pipeline_fn: Box::new(Render3d::default_pipeline),
        }
    }
}

impl<S: 'static> D3Node<S> {
    #[inline]
    pub fn new(render_fn: impl Fn(&mut S, &mut Render3dCtx) + 'static) -> Self {
        Self {
            meshes: Default::default(),
            camera_buffer: Default::default(),
            camera_bind_group: Default::default(),
            default_pipelines: Default::default(),
            render_fn: Box::new(render_fn),
            default_pipeline_fn: Box::new(default_pipeline::<S>),
        }
    }
}

impl<S> PassNode<S> for D3Node<S> {
    #[inline]
    fn run<'a>(&'a mut self, ctx: &mut PassNodeCtx<'_, 'a>, state: &mut S) {
        let mut render_3d_ctx = Render3dCtx {
            meshes: &mut self.meshes,
            render_ctx: &ctx.render_ctx,
        };

        (self.render_fn)(state, &mut render_3d_ctx);

        let sample_count = ctx.data.get::<SampleCount>().unwrap_or(&SampleCount(1));
        let format = ctx
            .data
            .get::<TargetFormat>()
            .cloned()
            .unwrap_or_else(|| TargetFormat(ctx.view.format))
            .0;
        let view_proj = ctx
            .data
            .get::<ViewProj>()
            .cloned()
            .unwrap_or_else(|| ViewProj(ctx.view.view_proj));

        let default_pipeline_fn = &self.default_pipeline_fn;
        let default_pipeline = self.default_pipelines.entry(format).or_insert_with(|| {
            default_pipeline_fn(state, &ctx.render_ctx.device, format, sample_count.0)
        });

        if let Some(ref buffer) = self.camera_buffer {
            ctx.render_ctx
                .queue
                .write_buffer(buffer, 0, bytes_of(&view_proj.0));
        } else {
            let buffer =
                ctx.render_ctx
                    .device
                    .create_buffer_init(&ike_wgpu::BufferInitDescriptor {
                        label: None,
                        contents: bytes_of(&ctx.view.view_proj),
                        usage: ike_wgpu::BufferUsages::UNIFORM | ike_wgpu::BufferUsages::COPY_DST,
                    });

            self.camera_buffer = Some(buffer);
        }

        if self.camera_bind_group.is_none() {
            let bind_group_layout =
                ctx.render_ctx
                    .device
                    .create_bind_group_layout(&ike_wgpu::BindGroupLayoutDescriptor {
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
                    });

            let bind_group = ctx
                .render_ctx
                .device
                .create_bind_group(&ike_wgpu::BindGroupDescriptor {
                    label: None,
                    layout: &bind_group_layout,
                    entries: &[ike_wgpu::BindGroupEntry {
                        binding: 0,
                        resource: self.camera_buffer.as_ref().unwrap().as_entire_binding(),
                    }],
                });

            self.camera_bind_group = Some(bind_group);
        }

        self.meshes.prepare(&ctx.render_ctx);

        ctx.render_pass.set_pipeline(default_pipeline);

        ctx.render_pass
            .set_bind_group(0, self.camera_bind_group.as_ref().unwrap(), &[]);

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
                instance_buffer
                    .buffer
                    .slice(..index_buffer.buffer.len as u64),
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

            instances.count = 0;
        }
    }
}
