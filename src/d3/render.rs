use std::collections::{BTreeMap, HashMap};

use bytemuck::{bytes_of, cast_slice};
use glam::{Mat4, Vec3};

use crate::{
    id::{HasId, Id},
    prelude::{Camera, Color, Texture},
    renderer::{Drawable, PassNode, PassNodeCtx, RenderCtx, SampleCount, TargetFormat},
    texture::TextureVersion,
};

use super::{
    default_pipeline::default_pipeline, BufferVersion, Indices, Mesh, PbrFlags, PbrMaterial,
    Skeleton, Transform3d, Vertices,
};

pub(crate) struct SizedBuffer {
    len: usize,
    pub(crate) buffer: ike_wgpu::Buffer,
    usage: ike_wgpu::BufferUsages,
}

impl SizedBuffer {
    #[inline]
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

    #[inline]
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
    filter_mode: ike_wgpu::FilterMode,
    albedo_texture: Option<Id<Texture>>,
    metallic_texture: Option<Id<Texture>>,
    normal_map: Option<Id<Texture>>,
}

#[derive(Default)]
struct Instances {
    count: u32,
    data: Vec<u8>,
    albedo_version: TextureVersion,
    metallic_roughness_version: TextureVersion,
    normal_map_version: TextureVersion,
    buffer: Option<SizedBuffer>,
    texture_bind_group: Option<ike_wgpu::BindGroup>,
    joint_matrices: Option<Id<Skeleton>>,
}

#[derive(Default)]
pub(crate) struct JointMatrices {
    pub joint_matrices: Vec<Mat4>,
    pub buffer: Option<SizedBuffer>,
    pub bind_group: Option<ike_wgpu::BindGroup>,
}

#[derive(Default)]
pub(crate) struct Meshes {
    vertex_buffers: BTreeMap<Id<Vertices>, VersionedBuffer>,
    index_buffers: BTreeMap<Id<Indices>, VersionedBuffer>,
    pub joint_matrices: BTreeMap<Id<Skeleton>, JointMatrices>,
    pub textures: BTreeMap<Id<Texture>, ike_wgpu::TextureView>,
    instances: HashMap<InstancesId, Instances>,
}

impl Meshes {
    #[inline]
    pub fn add_instance<V: bytemuck::Pod>(
        &mut self,
        ctx: &RenderCtx,
        mesh: &Mesh<V>,
        data: &[u8],
        filter_mode: ike_wgpu::FilterMode,
        joint_matrices: Option<Id<Skeleton>>,
        albedo_texture: Option<&Texture>,
        metallic_roughness_texture: Option<&Texture>,
        normal_map: Option<&Texture>,
    ) {
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

                buffer.version = mesh.indices.version();
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
            filter_mode,
            albedo_texture: albedo_texture.as_ref().map(|t| t.id()),
            metallic_texture: metallic_roughness_texture.as_ref().map(|t| t.id()),
            normal_map: normal_map.as_ref().map(|t| t.id()),
        };

        let instances = self.instances.entry(id).or_default();

        let len = instances.data.len();

        instances.count += 1;

        instances.data.resize(len + data.len(), 0);
        instances.data[len..].copy_from_slice(data);

        instances.joint_matrices = joint_matrices;

        if let Some(ref texture) = albedo_texture {
            let version = &mut instances.albedo_version;

            if texture.outdated(*version) {
                self.textures.insert(
                    texture.id(),
                    texture.texture(ctx).create_view(&Default::default()),
                );

                *version = texture.version();
                instances.texture_bind_group = None;
            }
        }

        if let Some(ref texture) = metallic_roughness_texture {
            let version = &mut instances.metallic_roughness_version;

            if texture.outdated(*version) {
                self.textures.insert(
                    texture.id(),
                    texture.texture(ctx).create_view(&Default::default()),
                );

                *version = texture.version();
                instances.texture_bind_group = None;
            }
        }

        if let Some(ref texture) = normal_map {
            let version = &mut instances.normal_map_version;

            if texture.outdated(*version) {
                self.textures.insert(
                    texture.id(),
                    texture.texture(ctx).create_view(&Default::default()),
                );

                *version = texture.version();
                instances.texture_bind_group = None;
            }
        }
    }

    #[inline]
    pub fn prepare(
        &mut self,
        ctx: &RenderCtx,
        textures_layout: &ike_wgpu::BindGroupLayout,
        default_texture: &ike_wgpu::TextureView,
    ) {
        for (id, instances) in &mut self.instances {
            // instance buffer

            if let Some(ref mut buffer) = instances.buffer {
                buffer.write(&ctx.device, &ctx.queue, &instances.data);
            } else {
                instances.buffer = Some(SizedBuffer::new(
                    &ctx.device,
                    &instances.data,
                    ike_wgpu::BufferUsages::VERTEX,
                ));
            }

            // textures

            if instances.texture_bind_group.is_none() {
                let sampler = ctx.device.create_sampler(&ike_wgpu::SamplerDescriptor {
                    address_mode_u: ike_wgpu::AddressMode::Repeat,
                    address_mode_v: ike_wgpu::AddressMode::Repeat,
                    address_mode_w: ike_wgpu::AddressMode::Repeat,
                    min_filter: id.filter_mode,
                    mag_filter: id.filter_mode,
                    ..Default::default()
                });

                let albedo = match id.albedo_texture {
                    Some(ref id) => self.textures.get(id).unwrap(),
                    None => default_texture,
                };

                let metallic = match id.metallic_texture {
                    Some(ref id) => self.textures.get(id).unwrap(),
                    None => default_texture,
                };

                let normal_map = match id.normal_map {
                    Some(ref id) => self.textures.get(id).unwrap(),
                    None => default_texture,
                };

                let bind_group = ctx
                    .device
                    .create_bind_group(&ike_wgpu::BindGroupDescriptor {
                        label: None,
                        layout: textures_layout,
                        entries: &[
                            ike_wgpu::BindGroupEntry {
                                binding: 0,
                                resource: ike_wgpu::BindingResource::Sampler(&sampler),
                            },
                            ike_wgpu::BindGroupEntry {
                                binding: 1,
                                resource: ike_wgpu::BindingResource::TextureView(albedo),
                            },
                            ike_wgpu::BindGroupEntry {
                                binding: 2,
                                resource: ike_wgpu::BindingResource::TextureView(metallic),
                            },
                            ike_wgpu::BindGroupEntry {
                                binding: 3,
                                resource: ike_wgpu::BindingResource::TextureView(normal_map),
                            },
                        ],
                    });

                instances.texture_bind_group = Some(bind_group);
            }
        }

        // joint matrices
        for (_id, joint_matrices) in &mut self.joint_matrices {
            let data: &[u8] = cast_slice(&joint_matrices.joint_matrices);

            if let Some(ref mut buffer) = joint_matrices.buffer {
                if buffer.len < data.len() {
                    joint_matrices.bind_group = None;
                }

                buffer.write(&ctx.device, &ctx.queue, data);
            } else {
                let buffer = SizedBuffer::new(
                    &ctx.device,
                    if data.len() == 0 { &[0; 16] } else { data },
                    ike_wgpu::BufferUsages::COPY_DST | ike_wgpu::BufferUsages::STORAGE,
                );

                joint_matrices.buffer = Some(buffer);
            }

            if joint_matrices.bind_group.is_none() {
                let layout =
                    ctx.device
                        .create_bind_group_layout(&ike_wgpu::BindGroupLayoutDescriptor {
                            label: None,
                            entries: &[ike_wgpu::BindGroupLayoutEntry {
                                binding: 0,
                                ty: ike_wgpu::BindingType::Buffer {
                                    ty: ike_wgpu::BufferBindingType::Storage { read_only: true },
                                    has_dynamic_offset: false,
                                    min_binding_size: None,
                                },
                                visibility: ike_wgpu::ShaderStages::VERTEX_FRAGMENT,
                                count: None,
                            }],
                        });

                let bind_group = ctx
                    .device
                    .create_bind_group(&ike_wgpu::BindGroupDescriptor {
                        label: None,
                        layout: &layout,
                        entries: &[ike_wgpu::BindGroupEntry {
                            binding: 0,
                            resource: joint_matrices
                                .buffer
                                .as_ref()
                                .unwrap()
                                .buffer
                                .as_entire_binding(),
                        }],
                    });

                joint_matrices.bind_group = Some(bind_group);
            }
        }
    }

    #[inline]
    pub fn clear(&mut self) {
        for (_, instances) in &mut self.instances {
            instances.data.clear();
            instances.count = 0;
        }

        for joint_matrices in self.joint_matrices.values_mut() {
            joint_matrices.joint_matrices.clear();
        }
    }
}

#[derive(Clone, Debug)]
pub struct PointLight {
    pub position: Vec3,
    pub color: Color,
    pub intensity: f32,
    pub range: f32,
    pub radius: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PointLightRaw {
    pub position: [f32; 4],
    pub color: [f32; 4],
    pub light_params: [f32; 4],
}

impl Drawable for PointLight {
    type Node = D3Node;

    #[inline]
    fn draw(&self, _ctx: &RenderCtx, node: &mut Self::Node) {
        let color = self.color * self.intensity;

        node.point_lights.push(PointLightRaw {
            position: [self.position.x, self.position.y, self.position.z, 0.0],
            color: color.into(),
            light_params: [1.0 / (self.range * self.range), self.radius, 0.0, 0.0],
        });
    }
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    pub view_proj: Mat4,
    pub camera_position: Vec3,
    pub point_light_count: u32,
    pub point_lights: [PointLightRaw; 64],
}

#[inline]
fn point_lights(lights: &[PointLightRaw]) -> [PointLightRaw; 64] {
    let mut point_lights: [PointLightRaw; 64] = bytemuck::Zeroable::zeroed();

    for (i, light) in lights.iter().enumerate() {
        point_lights[i] = *light;
    }

    point_lights
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct InstanceData {
    pub transform: Mat4,
    pub albedo: [f32; 4],
    pub roughness: f32,
    pub metallic: f32,
    pub reflectance: f32,
    pub flags: PbrFlags,
    pub joint_count: u32,
    pub emissive: [f32; 3],
}

impl Default for InstanceData {
    #[inline]
    fn default() -> Self {
        Self {
            transform: Mat4::IDENTITY,
            albedo: Color::WHITE.into(),
            roughness: 0.089,
            metallic: 0.01,
            reflectance: 0.5,
            flags: PbrFlags::EMPTY,
            joint_count: 0,
            emissive: [0.0; 3],
        }
    }
}

impl InstanceData {
    #[inline]
    pub fn new(transform: Mat4, material: &PbrMaterial, mut flags: PbrFlags) -> Self {
        if material.normal_map.is_some() {
            flags |= PbrFlags::NORMAL_MAP;
        }

        Self {
            transform,
            albedo: material.albedo.into(),
            roughness: material.roughness,
            metallic: material.metallic,
            reflectance: material.reflectance,
            flags,
            joint_count: 0,
            emissive: [
                material.emission.r,
                material.emission.g,
                material.emission.b,
            ],
        }
    }
}

impl Drawable for Mesh {
    type Node = D3Node;

    #[inline]
    fn draw(&self, ctx: &RenderCtx, node: &mut Self::Node) {
        node.meshes.add_instance(
            ctx,
            self,
            bytes_of(&InstanceData::default()),
            ike_wgpu::FilterMode::Linear,
            None,
            None,
            None,
            None,
        );
    }
}

impl Drawable for (&Mesh, &Transform3d) {
    type Node = D3Node;

    #[inline]
    fn draw(&self, ctx: &RenderCtx, node: &mut Self::Node) {
        node.meshes.add_instance(
            ctx,
            self.0,
            bytes_of(&InstanceData {
                transform: self.1.matrix(),
                ..Default::default()
            }),
            ike_wgpu::FilterMode::Linear,
            None,
            None,
            None,
            None,
        );
    }
}

impl Drawable for (&Mesh, &Transform3d, &PbrMaterial<'_>) {
    type Node = D3Node;

    #[inline]
    fn draw(&self, ctx: &RenderCtx, node: &mut Self::Node) {
        node.meshes.add_instance(
            ctx,
            self.0,
            bytes_of(&InstanceData::new(self.1.matrix(), self.2, PbrFlags::EMPTY)),
            self.2.filter_mode,
            None,
            self.2.albedo_texture.as_ref().map(AsRef::as_ref),
            self.2
                .metallic_roughness_texture
                .as_ref()
                .map(AsRef::as_ref),
            self.2.normal_map.as_ref().map(AsRef::as_ref),
        );
    }
}

impl Mesh {
    #[inline]
    pub fn transform<'a>(&'a self, transform: &'a Transform3d) -> (&Self, &Transform3d) {
        (self, transform)
    }

    #[inline]
    pub fn transform_material<'a>(
        &'a self,
        transform: &'a Transform3d,
        material: &'a PbrMaterial,
    ) -> (&Self, &Transform3d, &PbrMaterial) {
        (self, transform, material)
    }
}

pub struct D3Node {
    pub(crate) meshes: Meshes,
    pub(crate) point_lights: Vec<PointLightRaw>,
    textures_layout: Option<ike_wgpu::BindGroupLayout>,
    default_texture: Option<ike_wgpu::TextureView>,
    uniforms_buffer: Option<ike_wgpu::Buffer>,
    uniforms_bind_group: Option<ike_wgpu::BindGroup>,
    default_joint_matrices_bind_group: Option<ike_wgpu::BindGroup>,
    default_pipelines: HashMap<ike_wgpu::TextureFormat, ike_wgpu::RenderPipeline>,
}

impl Default for D3Node {
    #[inline]
    fn default() -> Self {
        Self {
            meshes: Default::default(),
            point_lights: Vec::new(),
            textures_layout: None,
            default_texture: None,
            uniforms_buffer: Default::default(),
            uniforms_bind_group: Default::default(),
            default_joint_matrices_bind_group: Default::default(),
            default_pipelines: Default::default(),
        }
    }
}

impl D3Node {
    #[inline]
    pub fn create_default_texture(&mut self, ctx: &RenderCtx) {
        if self.default_texture.is_none() {
            let texture = ctx.device.create_texture_with_data(
                &ctx.queue,
                &ike_wgpu::TextureDescriptor {
                    label: None,
                    size: ike_wgpu::Extent3d {
                        width: 1,
                        height: 1,
                        depth_or_array_layers: 1,
                    },
                    dimension: ike_wgpu::TextureDimension::D2,
                    mip_level_count: 1,
                    sample_count: 1,
                    format: ike_wgpu::TextureFormat::Rgba8UnormSrgb,
                    usage: ike_wgpu::TextureUsages::COPY_DST
                        | ike_wgpu::TextureUsages::TEXTURE_BINDING,
                },
                &[255; 4],
            );

            let texture_view = texture.create_view(&Default::default());

            self.default_texture = Some(texture_view);
        }
    }

    #[inline]
    pub fn create_default_joint_matrices_bind_group(&mut self, ctx: &RenderCtx) {
        if self.default_joint_matrices_bind_group.is_none() {
            let buffer = ctx
                .device
                .create_buffer_init(&ike_wgpu::BufferInitDescriptor {
                    label: None,
                    contents: &[0; 16],
                    usage: ike_wgpu::BufferUsages::COPY_DST | ike_wgpu::BufferUsages::STORAGE,
                });

            let layout =
                ctx.device
                    .create_bind_group_layout(&ike_wgpu::BindGroupLayoutDescriptor {
                        label: None,
                        entries: &[ike_wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            ty: ike_wgpu::BindingType::Buffer {
                                ty: ike_wgpu::BufferBindingType::Storage { read_only: true },
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            visibility: ike_wgpu::ShaderStages::VERTEX_FRAGMENT,
                            count: None,
                        }],
                    });

            let bind_group = ctx
                .device
                .create_bind_group(&ike_wgpu::BindGroupDescriptor {
                    label: None,
                    layout: &layout,
                    entries: &[ike_wgpu::BindGroupEntry {
                        binding: 0,
                        resource: buffer.as_entire_binding(),
                    }],
                });

            self.default_joint_matrices_bind_group = Some(bind_group);
        }
    }

    #[inline]
    pub fn create_textures_layout(&mut self, ctx: &RenderCtx) {
        if self.textures_layout.is_none() {
            let textures =
                ctx.device
                    .create_bind_group_layout(&ike_wgpu::BindGroupLayoutDescriptor {
                        label: None,
                        entries: &[
                            ike_wgpu::BindGroupLayoutEntry {
                                binding: 0,
                                ty: ike_wgpu::BindingType::Sampler {
                                    filtering: true,
                                    comparison: false,
                                },
                                visibility: ike_wgpu::ShaderStages::VERTEX_FRAGMENT,
                                count: None,
                            },
                            ike_wgpu::BindGroupLayoutEntry {
                                binding: 1,
                                ty: ike_wgpu::BindingType::Texture {
                                    sample_type: ike_wgpu::TextureSampleType::Float {
                                        filterable: true,
                                    },
                                    view_dimension: ike_wgpu::TextureViewDimension::D2,
                                    multisampled: false,
                                },
                                visibility: ike_wgpu::ShaderStages::VERTEX_FRAGMENT,
                                count: None,
                            },
                            ike_wgpu::BindGroupLayoutEntry {
                                binding: 2,
                                ty: ike_wgpu::BindingType::Texture {
                                    sample_type: ike_wgpu::TextureSampleType::Float {
                                        filterable: true,
                                    },
                                    view_dimension: ike_wgpu::TextureViewDimension::D2,
                                    multisampled: false,
                                },
                                visibility: ike_wgpu::ShaderStages::VERTEX_FRAGMENT,
                                count: None,
                            },
                            ike_wgpu::BindGroupLayoutEntry {
                                binding: 3,
                                ty: ike_wgpu::BindingType::Texture {
                                    sample_type: ike_wgpu::TextureSampleType::Float {
                                        filterable: true,
                                    },
                                    view_dimension: ike_wgpu::TextureViewDimension::D2,
                                    multisampled: false,
                                },
                                visibility: ike_wgpu::ShaderStages::VERTEX_FRAGMENT,
                                count: None,
                            },
                        ],
                    });

            self.textures_layout = Some(textures);
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
        self.create_default_texture(ctx.render_ctx);
        self.create_textures_layout(ctx.render_ctx);
        self.create_default_joint_matrices_bind_group(ctx.render_ctx);

        let sample_count = ctx.data.get::<SampleCount>().unwrap_or(&SampleCount(1));
        let format = ctx
            .data
            .get::<TargetFormat>()
            .cloned()
            .unwrap_or_else(|| TargetFormat(ctx.view.format))
            .0;
        let camera = ctx.data.get::<Camera>().unwrap_or_else(|| &ctx.view.camera);

        let textures_layout = self.textures_layout.as_ref().unwrap();
        let default_pipeline = self.default_pipelines.entry(format).or_insert_with(|| {
            default_pipeline(
                &ctx.render_ctx.device,
                textures_layout,
                format,
                sample_count.0,
            )
        });

        let uniforms = Uniforms {
            view_proj: camera.view_proj(),
            camera_position: camera.position,
            point_light_count: self.point_lights.len() as u32,
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

        self.meshes.prepare(
            &ctx.render_ctx,
            textures_layout,
            self.default_texture.as_ref().unwrap(),
        );

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

            ctx.render_pass
                .set_bind_group(1, instances.texture_bind_group.as_ref().unwrap(), &[]);

            if let Some(ref id) = instances.joint_matrices {
                ctx.render_pass.set_bind_group(
                    2,
                    self.meshes
                        .joint_matrices
                        .get(id)
                        .unwrap()
                        .bind_group
                        .as_ref()
                        .unwrap(),
                    &[],
                );
            } else {
                ctx.render_pass.set_bind_group(
                    2,
                    self.default_joint_matrices_bind_group.as_ref().unwrap(),
                    &[],
                );
            }

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
