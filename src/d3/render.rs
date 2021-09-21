use std::collections::{BTreeMap, HashMap};

use bytemuck::{bytes_of, cast_slice};
use glam::{Mat4, Vec3};

use crate::{cube_texture::CubeTexture, id::{HasId, Id}, prelude::{Camera, Color, HdrTexture, Texture}, renderer::{Drawable, PassNode, PassNodeCtx, RenderCtx, SampleCount, TargetFormat}};

use super::{
    default_pipeline::default_pipeline, BufferVersion, Indices, Mesh, PbrFlags, PbrMaterial,
    PbrMaterialRaw, Skeleton, Transform3d, Vertices,
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
    material: Id<PbrMaterial>,
}

#[derive(Default)]
struct Instances {
    count: u32,
    data: Vec<u8>,
    buffer: Option<SizedBuffer>,
    bind_group: Option<ike_wgpu::BindGroup>,
    joint_count: u32,
    joint_matrices: Option<Id<Skeleton>>,
    uniforms_buffer: Option<ike_wgpu::Buffer>,
}

struct Material {
    pub albedo_texture: Option<Id<Texture>>,
    pub metallic_roughness_texture: Option<Id<Texture>>,
    pub normal_map: Option<Id<Texture>>,
    pub raw: PbrMaterialRaw,
    pub filter_mode: ike_wgpu::FilterMode,
}

impl From<&PbrMaterial> for Material {
    #[inline]
    fn from(pbr_material: &PbrMaterial) -> Self {
        Self {
            albedo_texture: pbr_material.albedo_texture.as_ref().map(|t| t.id()),
            metallic_roughness_texture: pbr_material
                .metallic_roughness_texture
                .as_ref()
                .map(|t| t.id()),
            normal_map: pbr_material.normal_map.as_ref().map(|t| t.id()),
            raw: pbr_material.raw(),
            filter_mode: pbr_material.filter_mode,
        }
    }
}

#[derive(Default)]
pub(crate) struct JointMatrices {
    pub joint_matrices: Vec<Mat4>,
    pub buffer: Option<SizedBuffer>,
    pub bind_group: Option<ike_wgpu::BindGroup>,
}

#[derive(Default)]
pub(crate) struct Meshes {
    default_material: PbrMaterial,
    vertex_buffers: BTreeMap<Id<Vertices>, VersionedBuffer>,
    index_buffers: BTreeMap<Id<Indices>, VersionedBuffer>,
    textures: BTreeMap<Id<Texture>, ike_wgpu::TextureView>,
    materials: BTreeMap<Id<PbrMaterial>, Material>,
    joint_matrices: BTreeMap<Id<Skeleton>, JointMatrices>,
    instances: HashMap<InstancesId, Instances>,
}

impl Meshes {
    #[inline]
    pub fn register_texture(&mut self, ctx: &RenderCtx, texture: &Texture) {
        if !self.textures.contains_key(&texture.id()) {
            self.textures.insert(
                texture.id(),
                texture.texture(ctx).create_view(&Default::default()),
            );
        }
    }

    #[inline]
    pub fn register_material(&mut self, ctx: &RenderCtx, material: &PbrMaterial) {
        if !self.materials.contains_key(&material.id()) {
            if let Some(ref texture) = material.albedo_texture {
                self.register_texture(ctx, texture);
            }

            if let Some(ref texture) = material.metallic_roughness_texture {
                self.register_texture(ctx, texture);
            }

            if let Some(ref texture) = material.normal_map {
                self.register_texture(ctx, texture);
            }

            self.materials
                .insert(material.id(), Material::from(material));
        }
    }

    #[inline]
    pub fn register_joint_matrices(&mut self, id: Id<Skeleton>, matrices: &[Mat4]) {
        let joint_matrices = self.joint_matrices.entry(id).or_default();

        let len = joint_matrices.joint_matrices.len();

        joint_matrices
            .joint_matrices
            .resize(len + matrices.len(), Mat4::IDENTITY);
        joint_matrices.joint_matrices[len..].copy_from_slice(matrices);
    }

    #[inline]
    pub fn add_instance<V: bytemuck::Pod>(
        &mut self,
        ctx: &RenderCtx,
        mesh: &Mesh<V>,
        material: &Option<&PbrMaterial>,
        skeleton: Option<(Id<Skeleton>, u32)>,
        data: &[u8],
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

        let material_id = if let Some(material) = material {
            material.id()
        } else {
            self.default_material.id()
        };

        let id = InstancesId {
            vertex_buffer: mesh.vertices.id(),
            index_buffer: mesh.indices.id(),
            material: material_id,
        };

        let instances = self.instances.entry(id).or_default();

        let len = instances.data.len();

        instances.count += 1;

        instances.data.resize(len + data.len(), 0);
        instances.data[len..].copy_from_slice(data);

        if let Some((skeleton, joint_count)) = skeleton {
            instances.joint_matrices = Some(skeleton);
            instances.joint_count = joint_count;
        }
    }

    #[inline]
    pub fn prepare(
        &mut self,
        ctx: &RenderCtx,
        textures_layout: &ike_wgpu::BindGroupLayout,
        default_texture: &ike_wgpu::TextureView,
    ) {
        self.materials.insert(
            self.default_material.id(),
            Material::from(&self.default_material),
        );

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

            // material

            let material = &self.materials[&id.material];

            let mut flags = PbrFlags::EMPTY;

            if material.normal_map.is_some() {
                flags |= PbrFlags::NORMAL_MAP;
            }

            if instances.joint_matrices.is_some() {
                flags |= PbrFlags::SKINNED;
            }

            let mesh = MeshUniforms {
                material: material.raw,
                padding: [0; 4],
                flags,
                joint_count: instances.joint_count,
            };

            if let Some(ref buffer) = instances.uniforms_buffer {
                ctx.queue.write_buffer(buffer, 0, bytes_of(&mesh));
            } else {
                let buffer = ctx
                    .device
                    .create_buffer_init(&ike_wgpu::BufferInitDescriptor {
                        label: None,
                        contents: bytes_of(&mesh),
                        usage: ike_wgpu::BufferUsages::COPY_DST | ike_wgpu::BufferUsages::UNIFORM,
                    });

                instances.uniforms_buffer = Some(buffer);
            }

            // textures

            if instances.bind_group.is_none() {
                let sampler = ctx.device.create_sampler(&ike_wgpu::SamplerDescriptor {
                    address_mode_u: ike_wgpu::AddressMode::Repeat,
                    address_mode_v: ike_wgpu::AddressMode::Repeat,
                    address_mode_w: ike_wgpu::AddressMode::Repeat,
                    min_filter: material.filter_mode,
                    mag_filter: material.filter_mode,
                    ..Default::default()
                });

                let albedo = match material.albedo_texture {
                    Some(ref id) => self.textures.get(id).unwrap(),
                    None => default_texture,
                };

                let metallic = match material.metallic_roughness_texture {
                    Some(ref id) => self.textures.get(id).unwrap(),
                    None => default_texture,
                };

                let normal_map = match material.normal_map {
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
                            ike_wgpu::BindGroupEntry {
                                binding: 4,
                                resource: instances
                                    .uniforms_buffer
                                    .as_ref()
                                    .unwrap()
                                    .as_entire_binding(),
                            },
                        ],
                    });

                instances.bind_group = Some(bind_group);
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
    type Node = &'static mut D3Node;

    #[inline]
    fn draw(&self, _ctx: &RenderCtx, node: &mut D3Node) {
        let color = self.color * self.intensity;

        node.point_lights.push(PointLightRaw {
            position: [self.position.x, self.position.y, self.position.z, 0.0],
            color: color.into(),
            light_params: [1.0 / (self.range * self.range), self.radius, 0.0, 0.0],
        });
    }
}

#[derive(Clone, Debug)]
pub struct DirectionalLight {
    pub direction: Vec3,
    pub color: Color,
    pub illuminance: f32,
}

impl Default for DirectionalLight {
    #[inline]
    fn default() -> Self {
        Self {
            direction: -Vec3::Y,
            color: Color::WHITE,
            illuminance: 32000.0,
        }
    }
}

impl DirectionalLight {
    #[inline]
    pub fn intensity(&self) -> f32 {
        const APERTURE: f32 = 4.0;
        const SHUTTER_SPEED: f32 = 1.0 / 250.0;
        const SENSITIVITY: f32 = 100.0;
        let ev100 = f32::log2(APERTURE * APERTURE / SHUTTER_SPEED) - f32::log2(SENSITIVITY / 100.0);
        let exposure = 1.0 / (f32::powf(2.0, ev100) * 1.2);
        self.illuminance * exposure
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct DirectionalLightRaw {
    pub position: [f32; 4],
    pub direction: [f32; 4],
    pub color: [f32; 4],
    pub view_proj: [[f32; 4]; 4],
    pub near: f32,
    pub far: f32,
    pub size: [f32; 2],
}

impl Drawable for DirectionalLight {
    type Node = &'static mut D3Node;

    #[inline]
    fn draw(&self, _ctx: &RenderCtx, node: &mut D3Node) {
        node.directional_lights.push(self.clone());
    }
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    pub view_proj: [[f32; 4]; 4],
    pub camera_position: [f32; 4],
    pub point_lights: [PointLightRaw; 64],
    pub directional_lights: [DirectionalLightRaw; 16],
    pub point_light_count: u32,
    pub directional_light_count: u32,
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct MeshUniforms {
    material: PbrMaterialRaw,
    padding: [u8; 4],
    flags: PbrFlags,
    joint_count: u32,
}

#[inline]
fn point_lights(lights: &[PointLightRaw]) -> [PointLightRaw; 64] {
    let mut point_lights: [PointLightRaw; 64] = bytemuck::Zeroable::zeroed();

    for (i, light) in lights.iter().enumerate() {
        point_lights[i] = *light;
    }

    point_lights
}

impl Drawable for Mesh {
    type Node = &'static mut D3Node;

    #[inline]
    fn draw(&self, ctx: &RenderCtx, node: &mut D3Node) {
        node.meshes
            .add_instance(ctx, self, &None, None, bytes_of(&Mat4::IDENTITY));
    }
}

impl Drawable for (&Mesh, &Transform3d) {
    type Node = &'static mut D3Node;

    #[inline]
    fn draw(&self, ctx: &RenderCtx, node: &mut D3Node) {
        node.meshes
            .add_instance(ctx, self.0, &None, None, bytes_of(&self.1.matrix()));
    }
}

impl Drawable for (&Mesh, &Transform3d, &PbrMaterial) {
    type Node = &'static mut D3Node;

    #[inline]
    fn draw(&self, ctx: &RenderCtx, node: &mut D3Node) {
        node.meshes.register_material(ctx, self.2);

        node.meshes
            .add_instance(ctx, self.0, &Some(self.2), None, bytes_of(&self.1.matrix()));
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

#[derive(Default)]
pub struct D3Node {
    pub(crate) meshes: Meshes,
    pub(crate) point_lights: Vec<PointLightRaw>,
    pub(crate) directional_lights: Vec<DirectionalLight>,
    pub(crate) textures_layout: Option<ike_wgpu::BindGroupLayout>,
    default_texture: Option<ike_wgpu::TextureView>,
    pub(crate) env_texture: Option<ike_wgpu::TextureView>,
    pub(crate) env_texture_id: Option<Id<CubeTexture>>,
    uniforms_buffer: Option<ike_wgpu::Buffer>,
    uniforms_bind_group: Option<ike_wgpu::BindGroup>,
    default_joint_matrices_bind_group: Option<ike_wgpu::BindGroup>,
    default_pipelines: HashMap<ike_wgpu::TextureFormat, ike_wgpu::RenderPipeline>,
    pub(crate) depth_textures: Option<ike_wgpu::Texture>,
    pub(crate) depth_view_buffer: Option<ike_wgpu::Buffer>,
    pub(crate) depth_bind_group: Option<ike_wgpu::BindGroup>,
    pub(crate) depth_pipeline: Option<ike_wgpu::RenderPipeline>,
    pub(crate) shadow_map_layout: Option<ike_wgpu::BindGroupLayout>,
    pub(crate) shadow_map_bind_group: Option<ike_wgpu::BindGroup>,
}

impl D3Node {
    #[inline]
    pub fn set_env_texture(&mut self, ctx: &RenderCtx, texture: &CubeTexture) {
        if self.env_texture_id != Some(texture.id()) {
            self.env_texture = Some(texture.view(ctx));
            self.uniforms_bind_group = None; 
        }
    }

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
                            ike_wgpu::BindGroupLayoutEntry {
                                binding: 4,
                                ty: ike_wgpu::BindingType::Buffer {
                                    ty: ike_wgpu::BufferBindingType::Uniform,
                                    has_dynamic_offset: false,
                                    min_binding_size: None,
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
        self.directional_lights.clear();
    }

    #[inline]
    fn run<'a>(&'a mut self, ctx: &mut PassNodeCtx<'_, 'a>, _: &mut S) {
        self.create_default_texture(ctx.render_ctx);
        self.create_textures_layout(ctx.render_ctx);
        self.create_default_joint_matrices_bind_group(ctx.render_ctx);
        self.create_depth_pipeline(&ctx.render_ctx.device);

        let sample_count = ctx.data.get::<SampleCount>().unwrap_or(&SampleCount(1));
        let format = ctx
            .data
            .get::<TargetFormat>()
            .cloned()
            .unwrap_or_else(|| TargetFormat(ctx.view.format))
            .0;
        let camera = ctx.data.get::<Camera>().unwrap_or_else(|| &ctx.view.camera);

        let textures_layout = self.textures_layout.as_ref().unwrap();
        let shadow_map_layout = self.shadow_map_layout.as_ref().unwrap();
        let default_pipeline = self.default_pipelines.entry(format).or_insert_with(|| {
            default_pipeline(
                &ctx.render_ctx.device,
                textures_layout,
                shadow_map_layout,
                format,
                sample_count.0,
            )
        });

        self.meshes.prepare(
            &ctx.render_ctx,
            textures_layout,
            self.default_texture.as_ref().unwrap(),
        );

        let depth_view_buffer = self.depth_view_buffer.as_ref().unwrap();
        let depth_bind_group = self.depth_bind_group.as_ref().unwrap();
        let depth_textures = self.depth_textures.as_ref().unwrap();
        let depth_pipeline = self.depth_pipeline.as_ref().unwrap();
        let mut encoder = ctx
            .render_ctx
            .device
            .create_command_encoder(&Default::default());

        let mut directional_lights: [DirectionalLightRaw; 16] = bytemuck::Zeroable::zeroed();

        for (i, directional_light) in self.directional_lights.iter().enumerate() {
            let mut transform = Transform3d::from_translation(camera.position);
            transform.look_at(transform.translation + directional_light.direction, Vec3::Y);

            let view = Mat4::orthographic_rh(-35.0, 35.0, -35.0, 35.0, -500.0, 500.0);
            let view_proj = view * transform.matrix().inverse();

            let raw = DirectionalLightRaw {
                position: transform.translation.extend(0.0).into(),
                direction: directional_light.direction.extend(0.0).into(),
                color: (directional_light.color * directional_light.intensity()).into(),
                view_proj: view_proj.to_cols_array_2d(),
                near: -500.0,
                far: 500.0,
                size: [70.0, 70.0],
            };

            directional_lights[i] = raw;

            ctx.render_ctx
                .queue
                .write_buffer(depth_view_buffer, 0, bytes_of(&view_proj));

            let texture_view = depth_textures.create_view(&ike_wgpu::TextureViewDescriptor {
                label: None,
                base_array_layer: i as u32,
                ..Default::default()
            });

            let mut render_pass = encoder.begin_render_pass(&ike_wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[],
                depth_stencil_attachment: Some(ike_wgpu::RenderPassDepthStencilAttachment {
                    view: &texture_view,
                    depth_ops: Some(ike_wgpu::Operations {
                        load: ike_wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            render_pass.set_pipeline(depth_pipeline);

            for (id, instances) in &mut self.meshes.instances {
                if instances.count == 0 {
                    continue;
                }

                render_pass.set_bind_group(0, depth_bind_group, &[]);

                render_pass.set_bind_group(1, instances.bind_group.as_ref().unwrap(), &[]);

                if let Some(ref id) = instances.joint_matrices {
                    render_pass.set_bind_group(
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
                    render_pass.set_bind_group(
                        2,
                        self.default_joint_matrices_bind_group.as_ref().unwrap(),
                        &[],
                    );
                }

                let vertex_buffer = self.meshes.vertex_buffers.get(&id.vertex_buffer).unwrap();
                let index_buffer = self.meshes.index_buffers.get(&id.index_buffer).unwrap();

                render_pass.set_vertex_buffer(
                    0,
                    vertex_buffer
                        .buffer
                        .buffer
                        .slice(..vertex_buffer.buffer.len as u64),
                );

                let instance_buffer = instances.buffer.as_ref().unwrap();

                render_pass.set_vertex_buffer(
                    1,
                    instance_buffer.buffer.slice(..instance_buffer.len as u64),
                );

                render_pass.set_index_buffer(
                    index_buffer
                        .buffer
                        .buffer
                        .slice(..index_buffer.buffer.len as u64),
                    ike_wgpu::IndexFormat::Uint32,
                );

                render_pass.draw_indexed(
                    0..index_buffer.buffer.len as u32 / 4,
                    0,
                    0..instances.count,
                );
            }
        }

        ctx.render_ctx
            .queue
            .submit(std::iter::once(encoder.finish()));

        let uniforms = Uniforms {
            view_proj: camera.view_proj().to_cols_array_2d(),
            camera_position: camera.position.extend(0.0).into(),
            point_light_count: self.point_lights.len() as u32,
            point_lights: point_lights(&self.point_lights),
            directional_light_count: self.directional_lights.len() as u32,
            directional_lights,
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
                    entries: &[
                        ike_wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            ty: ike_wgpu::BindingType::Buffer {
                                ty: ike_wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            visibility: ike_wgpu::ShaderStages::VERTEX_FRAGMENT,
                            count: None,
                        },
                        ike_wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            ty: ike_wgpu::BindingType::Texture {
                                sample_type: ike_wgpu::TextureSampleType::Float {
                                    filterable: false,
                                },
                                view_dimension: ike_wgpu::TextureViewDimension::D2,
                                multisampled: false,
                            },
                            visibility: ike_wgpu::ShaderStages::VERTEX_FRAGMENT,
                            count: None,
                        },
                        ike_wgpu::BindGroupLayoutEntry {
                            binding: 2,
                            ty: ike_wgpu::BindingType::Sampler {
                                filtering: true,
                                comparison: false,
                            },
                            visibility: ike_wgpu::ShaderStages::VERTEX_FRAGMENT,
                            count: None,
                        },
                    ],
                },
            );

            let env_texture = if let Some(ref texture) = self.env_texture {
                texture
            } else {
                self.default_texture.as_ref().unwrap()
            };

            let bind_group =
                ctx.render_ctx
                    .device
                    .create_bind_group(&ike_wgpu::BindGroupDescriptor {
                        label: None,
                        layout: &bind_group_layout,
                        entries: &[
                            ike_wgpu::BindGroupEntry {
                                binding: 0,
                                resource: self
                                    .uniforms_buffer
                                    .as_ref()
                                    .unwrap()
                                    .as_entire_binding(),
                            },
                            ike_wgpu::BindGroupEntry {
                                binding: 1,
                                resource: ike_wgpu::BindingResource::TextureView(env_texture),
                            },
                            ike_wgpu::BindGroupEntry {
                                binding: 2,
                                resource: ike_wgpu::BindingResource::Sampler(
                                    &ctx.render_ctx.device.create_sampler(&Default::default()),
                                ),
                            },
                        ],
                    });

            self.uniforms_bind_group = Some(bind_group);
        }

        ctx.render_pass.set_pipeline(default_pipeline);

        ctx.render_pass
            .set_bind_group(0, self.uniforms_bind_group.as_ref().unwrap(), &[]);

        for (id, instances) in &mut self.meshes.instances {
            if instances.count == 0 {
                continue;
            }

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
                .set_bind_group(1, instances.bind_group.as_ref().unwrap(), &[]);

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

            ctx.render_pass
                .set_bind_group(3, self.shadow_map_bind_group.as_ref().unwrap(), &[]);

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
