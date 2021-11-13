use std::{collections::HashMap, num::NonZeroU32};

use bytemuck::{bytes_of, cast_slice, Zeroable};
use glam::{Mat4, Quat, Vec3};
use ike_assets::{Assets, Handle};
use ike_core::WorldRef;
use ike_render::*;
use ike_transform::{GlobalTransform, Transform};

use crate::{DirectionalLight, PbrMaterial, PointLight};

struct ShaderResources {
    shader: wgpu::ShaderModule,
    depth_group: wgpu::BindGroupLayout,
    group_0: wgpu::BindGroupLayout,
    group_1: wgpu::BindGroupLayout,
    group_2: wgpu::BindGroupLayout,
    group_3: wgpu::BindGroupLayout,
    layout: wgpu::PipelineLayout,
    default_tex: wgpu::TextureView,
    default_cube: wgpu::TextureView,
    sampler: wgpu::Sampler,
    pipelines: HashMap<RenderTarget, wgpu::RenderPipeline>,
    depth_pipeline: wgpu::RenderPipeline,
}

impl Default for ShaderResources {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl ShaderResources {
    #[inline]
    pub fn new() -> Self {
        let device = render_device();

        let shader = device.create_shader_module(&wgpu::include_wgsl!("pbr.wgsl"));
        let depth_shader = device.create_shader_module(&wgpu::include_wgsl!("depth.wgsl"));

        let depth_group = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                count: None,
            }],
        });

        let group_0 = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::Cube,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::Cube,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Sampler {
                        filtering: true,
                        comparison: false,
                    },
                    count: None,
                },
            ],
        });

        let group_1 = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Sampler {
                        filtering: true,
                        comparison: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let group_2 = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[],
        });

        let group_3 = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Depth,
                        view_dimension: wgpu::TextureViewDimension::D2Array,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Sampler {
                        filtering: true,
                        comparison: false,
                    },
                    count: None,
                },
            ],
        });

        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&group_0, &group_1, &group_2, &group_3],
            push_constant_ranges: &[],
        });

        let default_tex = device.create_texture_with_data(
            render_queue(),
            &wgpu::TextureDescriptor {
                label: None,
                size: wgpu::Extent3d {
                    width: 1,
                    height: 1,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            },
            &[255; 4],
        );

        let default_cube = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 6,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba32Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING,
        });

        let default_cube = default_cube.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::Cube),
            ..Default::default()
        });

        let depth_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&depth_group, &group_1],
            push_constant_ranges: &[],
        });

        let depth_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("depth_pipeline"),
            layout: Some(&depth_layout),
            vertex: wgpu::VertexState {
                module: &depth_shader,
                entry_point: "main",
                buffers: &[
                    wgpu::VertexBufferLayout {
                        array_stride: 12,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &[wgpu::VertexAttribute {
                            offset: 0,
                            format: wgpu::VertexFormat::Float32x3,
                            shader_location: 0,
                        }],
                    },
                    wgpu::VertexBufferLayout {
                        array_stride: 64,
                        step_mode: wgpu::VertexStepMode::Instance,
                        attributes: &[
                            wgpu::VertexAttribute {
                                offset: 0,
                                format: wgpu::VertexFormat::Float32x4,
                                shader_location: 8,
                            },
                            wgpu::VertexAttribute {
                                offset: 16,
                                format: wgpu::VertexFormat::Float32x4,
                                shader_location: 9,
                            },
                            wgpu::VertexAttribute {
                                offset: 32,
                                format: wgpu::VertexFormat::Float32x4,
                                shader_location: 10,
                            },
                            wgpu::VertexAttribute {
                                offset: 48,
                                format: wgpu::VertexFormat::Float32x4,
                                shader_location: 11,
                            },
                        ],
                    },
                ],
            },
            fragment: None,
            primitive: wgpu::PrimitiveState::default(),
            multisample: wgpu::MultisampleState::default(),
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::LessEqual,
                stencil: Default::default(),
                bias: Default::default(),
            }),
        });

        Self {
            shader,
            depth_group,
            group_0,
            group_1,
            group_2,
            group_3,
            layout,
            default_tex: default_tex.create_view(&Default::default()),
            default_cube: default_cube,
            sampler: device.create_sampler(&Default::default()),
            pipelines: HashMap::new(),
            depth_pipeline,
        }
    }

    fn create_pipeline(&mut self, target: RenderTarget) {
        if !self.pipelines.contains_key(&target) {
            let pipeline =
                render_device().create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("pbr pipeline"),
                    layout: Some(&self.layout),
                    vertex: wgpu::VertexState {
                        module: &self.shader,
                        entry_point: "main",
                        buffers: &[
                            wgpu::VertexBufferLayout {
                                array_stride: 12,
                                step_mode: wgpu::VertexStepMode::Vertex,
                                attributes: &[wgpu::VertexAttribute {
                                    offset: 0,
                                    format: wgpu::VertexFormat::Float32x3,
                                    shader_location: 0,
                                }],
                            },
                            wgpu::VertexBufferLayout {
                                array_stride: 12,
                                step_mode: wgpu::VertexStepMode::Vertex,
                                attributes: &[wgpu::VertexAttribute {
                                    offset: 0,
                                    format: wgpu::VertexFormat::Float32x3,
                                    shader_location: 1,
                                }],
                            },
                            wgpu::VertexBufferLayout {
                                array_stride: 8,
                                step_mode: wgpu::VertexStepMode::Vertex,
                                attributes: &[wgpu::VertexAttribute {
                                    offset: 0,
                                    format: wgpu::VertexFormat::Float32x2,
                                    shader_location: 2,
                                }],
                            },
                            wgpu::VertexBufferLayout {
                                array_stride: 16,
                                step_mode: wgpu::VertexStepMode::Vertex,
                                attributes: &[wgpu::VertexAttribute {
                                    offset: 0,
                                    format: wgpu::VertexFormat::Float32x4,
                                    shader_location: 3,
                                }],
                            },
                            wgpu::VertexBufferLayout {
                                array_stride: 16,
                                step_mode: wgpu::VertexStepMode::Vertex,
                                attributes: &[wgpu::VertexAttribute {
                                    offset: 0,
                                    format: wgpu::VertexFormat::Float32x4,
                                    shader_location: 4,
                                }],
                            },
                            wgpu::VertexBufferLayout {
                                array_stride: 64,
                                step_mode: wgpu::VertexStepMode::Instance,
                                attributes: &[
                                    wgpu::VertexAttribute {
                                        offset: 0,
                                        format: wgpu::VertexFormat::Float32x4,
                                        shader_location: 8,
                                    },
                                    wgpu::VertexAttribute {
                                        offset: 16,
                                        format: wgpu::VertexFormat::Float32x4,
                                        shader_location: 9,
                                    },
                                    wgpu::VertexAttribute {
                                        offset: 32,
                                        format: wgpu::VertexFormat::Float32x4,
                                        shader_location: 10,
                                    },
                                    wgpu::VertexAttribute {
                                        offset: 48,
                                        format: wgpu::VertexFormat::Float32x4,
                                        shader_location: 11,
                                    },
                                ],
                            },
                        ],
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &self.shader,
                        entry_point: "main",
                        targets: &[wgpu::ColorTargetState {
                            format: target.format,
                            blend: None,
                            write_mask: wgpu::ColorWrites::ALL,
                        }],
                    }),
                    primitive: wgpu::PrimitiveState::default(),
                    multisample: wgpu::MultisampleState {
                        count: target.samples,
                        ..Default::default()
                    },
                    depth_stencil: Some(wgpu::DepthStencilState {
                        format: wgpu::TextureFormat::Depth32Float,
                        depth_write_enabled: true,
                        depth_compare: wgpu::CompareFunction::LessEqual,
                        stencil: Default::default(),
                        bias: Default::default(),
                    }),
                });

            self.pipelines.insert(target, pipeline);
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
struct InstanceId {
    material: Handle<PbrMaterial>,
    mesh: Handle<Mesh>,
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct PointLightRaw {
    position: [f32; 4],
    color: [f32; 4],
    params: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct DirectionalLightRaw {
    position: [f32; 4],
    direction: [f32; 4],
    color: [f32; 4],
    view_proj: [[f32; 4]; 4],
    near: f32,
    far: f32,
    size: [f32; 2],
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct UniformsRaw {
    view_proj: [[f32; 4]; 4],
    camera_position: [f32; 4],
    point_lights: [PointLightRaw; 64],
    directional_lights: [DirectionalLightRaw; 16],
    point_light_count: u32,
    directional_light_count: u32,
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct MaterialRaw {
    albedo: [f32; 4],
    emission: [f32; 4],
    roughness: f32,
    metallic: f32,
    reflectance: f32,
    shadow_softness: f32,
    shadow_softness_falloff: f32,
    shadow_block_samples: u32,
    shadow_pcf_samples: u32,
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct MeshRaw {
    material: MaterialRaw,
    padding: [u8; 4],
    flags: u32,
    joint_count: u32,
}

struct MaterialResources {
    mesh_buffer: wgpu::Buffer,
    group_1: wgpu::BindGroup,
    group_2: wgpu::BindGroup,
    group_3: wgpu::BindGroup,
}

impl MaterialResources {
    #[inline]
    fn new(
        material: &PbrMaterial,
        shadows: &wgpu::TextureView,
        resources: &ShaderResources,
        textures: &Assets<Texture>,
    ) -> Self {
        let device = render_device();

        let mut mesh = MeshRaw {
            material: MaterialRaw {
                albedo: material.albedo.into(),
                emission: material.emission.into(),
                roughness: material.roughness,
                metallic: material.metallic,
                reflectance: material.reflectance,
                shadow_softness: material.shadow_softness,
                shadow_softness_falloff: material.shadow_softness_falloff,
                shadow_block_samples: material.shadow_blocker_samples,
                shadow_pcf_samples: material.shadow_pcf_samples,
            },
            padding: [0; 4],
            flags: 0,
            joint_count: 0,
        };

        let albedo_texture;

        let albedo_texture = if let Some(ref handle) = material.albedo_texture {
            let texture = textures.get(handle).unwrap();

            albedo_texture = texture.texture().create_view(&Default::default());

            &albedo_texture
        } else {
            &resources.default_tex
        };

        let metallic_roughness_texture;

        let metallic_roughness_texture =
            if let Some(ref handle) = material.metallic_roughness_texture {
                let texture = textures.get(handle).unwrap();

                metallic_roughness_texture = texture.texture().create_view(&Default::default());

                &metallic_roughness_texture
            } else {
                &resources.default_tex
            };

        let normal_map;

        let normal_map = if let Some(ref handle) = material.normal_map {
            let texture = textures.get(handle).unwrap();

            normal_map = texture.texture().create_view(&Default::default());

            mesh.flags |= 0b1;

            &normal_map
        } else {
            &resources.default_tex
        };

        let mesh_buffer = device.create_buffer_init(&wgpu::BufferInitDescriptor {
            label: None,
            contents: bytes_of(&mesh),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let group_1 = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &resources.group_1,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Sampler(&resources.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(albedo_texture),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(metallic_roughness_texture),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(normal_map),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: mesh_buffer.as_entire_binding(),
                },
            ],
        });

        let group_2 = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &resources.group_2,
            entries: &[],
        });

        let group_3 = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &resources.group_3,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&shadows),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&resources.sampler),
                },
            ],
        });

        Self {
            mesh_buffer,
            group_1,
            group_2,
            group_3,
        }
    }

    #[inline]
    pub fn update(&mut self, material: &PbrMaterial) {
        let mut flags = 0;

        if material.normal_map.is_some() {
            flags |= 0b1;
        }

        let mesh = MeshRaw {
            material: MaterialRaw {
                albedo: material.albedo.into(),
                emission: material.emission.into(),
                roughness: material.roughness,
                metallic: material.metallic,
                reflectance: material.reflectance,
                shadow_softness: material.shadow_softness,
                shadow_softness_falloff: material.shadow_softness_falloff,
                shadow_block_samples: material.shadow_blocker_samples,
                shadow_pcf_samples: material.shadow_pcf_samples,
            },
            padding: [0; 4],
            flags,
            joint_count: 0,
        };

        render_queue().write_buffer(&self.mesh_buffer, 0, bytes_of(&mesh));
    }
}

#[derive(Default)]
pub struct PbrNode {
    shadows: Option<wgpu::Texture>,
    uniforms: Option<wgpu::Buffer>,
    uniforms_group: Option<wgpu::BindGroup>,
    current_env: Option<Handle<Environment>>,
    materials: HashMap<Handle<PbrMaterial>, MaterialResources>,
    instances: HashMap<InstanceId, Buffer>,
    view_buffers: HashMap<usize, (wgpu::Buffer, wgpu::BindGroup)>,
}

impl PbrNode {
    pub const TARGET: &'static str = "target";
    pub const DEPTH: &'static str = "depth";
    pub const CAMERA: &'static str = "camera";
}

impl RenderNode for PbrNode {
    fn input(&self) -> Vec<EdgeSlotInfo> {
        vec![
            EdgeSlotInfo::new::<RenderTexture>(Self::TARGET),
            EdgeSlotInfo::new::<wgpu::Texture>(Self::DEPTH),
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

        let mut resources = world.get_resource_mut::<ShaderResources>().unwrap();
        resources.create_pipeline(target.target());

        let view = target.texture().create_view(&Default::default());

        let mut instances: HashMap<_, Vec<[[f32; 4]; 4]>> = HashMap::new();

        for (transform, material, mesh) in world
            .query::<(&GlobalTransform, &Handle<PbrMaterial>, &Handle<Mesh>)>()
            .unwrap()
        {
            let id = InstanceId {
                material: material.clone(),
                mesh: mesh.clone(),
            };

            instances
                .entry(id)
                .or_insert_with(Default::default)
                .push(transform.matrix().to_cols_array_2d());
        }

        let mut point_lights = [PointLightRaw::zeroed(); 64];
        let mut point_light_count = 0;

        for (transform, point_light) in world.query::<(&GlobalTransform, &PointLight)>().unwrap() {
            point_lights[point_light_count] = PointLightRaw {
                position: transform.translation.extend(0.0).into(),
                color: point_light.color.into(),
                params: [
                    1.0 / (point_light.range * point_light.range),
                    point_light.radius,
                    0.0,
                    0.0,
                ],
            };

            point_light_count += 1;
        }

        if self.shadows.is_none() {
            let shadows = render_device().create_texture(&wgpu::TextureDescriptor {
                label: None,
                size: wgpu::Extent3d {
                    width: 4096,
                    height: 4096,
                    depth_or_array_layers: 16,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Depth32Float,
                usage: wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::RENDER_ATTACHMENT,
            });

            self.shadows = Some(shadows);
        }

        let mut meshes = world.get_resource_mut::<Assets<Mesh>>().unwrap();
        let materials = world.get_resource::<Assets<PbrMaterial>>().unwrap();

        for (id, material) in &mut self.materials {
            material.update(materials.get(id).unwrap());
        }

        let textures = world.get_resource::<Assets<Texture>>().unwrap();

        for (id, instances) in &instances {
            if !self.materials.contains_key(&id.material) {
                let material = materials.get(&id.material).unwrap();

                let view = self
                    .shadows
                    .as_ref()
                    .unwrap()
                    .create_view(&Default::default());

                let material = MaterialResources::new(material, &view, &resources, &textures);

                self.materials.insert(id.material.clone(), material);
            }

            let instance_buffer = self
                .instances
                .entry(id.clone())
                .or_insert_with(|| Buffer::new(wgpu::BufferUsages::VERTEX));
            instance_buffer.write(cast_slice(instances));

            instance_buffer.raw();

            let mesh = meshes.get_mut(&id.mesh).unwrap();

            mesh.index_buffer().raw();

            mesh.buffer(Mesh::POSITION).unwrap().raw();
            mesh.buffer(Mesh::NORMAL).unwrap().raw();
            mesh.buffer(Mesh::UV).unwrap().raw();
            mesh.buffer(Mesh::TANGENT).unwrap().raw();
            mesh.buffer(Mesh::COLOR).unwrap().raw();
        }

        let camera = input.get::<Camera>(Self::CAMERA).unwrap();

        let mut directional_lights = [DirectionalLightRaw::zeroed(); 16];
        let mut directional_light_count = 0;

        for light in world.query::<&DirectionalLight>().unwrap() {
            let mut transform = Transform::from_translation(camera.position);
            transform.rotation =
                Quat::from_rotation_arc_colinear(-Vec3::Z, light.direction.normalize());

            let view_proj = Mat4::orthographic_rh(-35.0, 35.0, -35.0, 35.0, -500.0, 500.0)
                * transform.matrix().inverse();

            let directional_light = DirectionalLightRaw {
                position: camera.position.extend(0.0).into(),
                direction: light.direction.normalize().extend(0.0).into(),
                color: light.color.into(),
                view_proj: view_proj.to_cols_array_2d(),
                near: -500.0,
                far: 500.0,
                size: [70.0, 70.0],
            };

            let view = self
                .shadows
                .as_ref()
                .unwrap()
                .create_view(&wgpu::TextureViewDescriptor {
                    base_array_layer: directional_light_count as u32,
                    array_layer_count: Some(NonZeroU32::new(1).unwrap()),
                    ..Default::default()
                });

            let bind_group = if let Some((buffer, group)) =
                self.view_buffers.get(&directional_light_count)
            {
                render_queue().write_buffer(buffer, 0, bytes_of(&view_proj));

                group
            } else {
                let buffer = render_device().create_buffer_init(&wgpu::BufferInitDescriptor {
                    label: None,
                    contents: bytes_of(&view_proj),
                    usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
                });

                let bind_group = render_device().create_bind_group(&wgpu::BindGroupDescriptor {
                    label: None,
                    layout: &resources.depth_group,
                    entries: &[wgpu::BindGroupEntry {
                        binding: 0,
                        resource: buffer.as_entire_binding(),
                    }],
                });

                self.view_buffers
                    .insert(directional_light_count, (buffer, bind_group));

                &self.view_buffers[&directional_light_count].1
            };

            directional_lights[directional_light_count] = directional_light;
            directional_light_count += 1;

            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            render_pass.set_pipeline(&resources.depth_pipeline);

            render_pass.set_bind_group(0, bind_group, &[]);

            for (id, instances) in &instances {
                let mesh_group = &self.materials[&id.material];

                render_pass.set_bind_group(1, &mesh_group.group_1, &[]);

                let mesh = meshes.get(&id.mesh).unwrap();

                render_pass
                    .set_vertex_buffer(0, mesh.get_raw_buffer(Mesh::POSITION).unwrap().slice(..));
                render_pass.set_vertex_buffer(1, self.instances[id].get_raw().unwrap().slice(..));

                render_pass.set_index_buffer(
                    mesh.get_index_buffer_raw().unwrap().slice(..),
                    wgpu::IndexFormat::Uint32,
                );

                render_pass.draw_indexed(
                    0..mesh.indices().len() as u32,
                    0,
                    0..instances.len() as u32,
                );
            }
        }

        let uniforms = UniformsRaw {
            view_proj: camera.view_proj().to_cols_array_2d(),
            camera_position: camera.position.extend(0.0).into(),
            point_lights,
            directional_lights,
            point_light_count: point_light_count as u32,
            directional_light_count: directional_light_count as u32,
        };

        if let Some(ref buffer) = self.uniforms {
            render_queue().write_buffer(buffer, 0, bytes_of(&uniforms));
        } else {
            let buffer = render_device().create_buffer_init(&wgpu::BufferInitDescriptor {
                label: None,
                contents: bytes_of(&uniforms),
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
            });

            self.uniforms = Some(buffer);
        }

        let env = world.get_resource::<Handle<Environment>>();

        if self.uniforms_group.is_none() || env.as_deref() != self.current_env.as_ref() {
            let group = if let Some(env) = env {
                let envs = world.get_resource::<Assets<Environment>>().unwrap();
                let env = envs.get(&env).unwrap();

                render_device().create_bind_group(&wgpu::BindGroupDescriptor {
                    label: None,
                    layout: &resources.group_0,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: self.uniforms.as_ref().unwrap().as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::TextureView(&env.env_texture.view()),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: wgpu::BindingResource::TextureView(
                                &env.irradiance_texture.view(),
                            ),
                        },
                        wgpu::BindGroupEntry {
                            binding: 3,
                            resource: wgpu::BindingResource::Sampler(&resources.sampler),
                        },
                    ],
                })
            } else {
                render_device().create_bind_group(&wgpu::BindGroupDescriptor {
                    label: None,
                    layout: &resources.group_0,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: self.uniforms.as_ref().unwrap().as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::TextureView(&resources.default_cube),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: wgpu::BindingResource::TextureView(&resources.default_cube),
                        },
                        wgpu::BindGroupEntry {
                            binding: 3,
                            resource: wgpu::BindingResource::Sampler(&resources.sampler),
                        },
                    ],
                })
            };

            self.uniforms_group = Some(group);
        }

        let depth = input.get::<wgpu::TextureView>(Self::DEPTH).unwrap();

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
                    load: wgpu::LoadOp::Clear(1.0),
                    store: true,
                }),
                stencil_ops: None,
            }),
        });

        render_pass.set_pipeline(&resources.pipelines[&target.target()]);

        for (id, instances) in instances {
            let mesh = meshes.get(&id.mesh).unwrap();
            let material = self.materials.get(&id.material).unwrap();

            render_pass.set_bind_group(0, self.uniforms_group.as_ref().unwrap(), &[]);
            render_pass.set_bind_group(1, &material.group_1, &[]);
            render_pass.set_bind_group(2, &material.group_2, &[]);
            render_pass.set_bind_group(3, &material.group_3, &[]);

            render_pass.set_index_buffer(
                mesh.get_index_buffer_raw().unwrap().slice(..),
                wgpu::IndexFormat::Uint32,
            );

            render_pass
                .set_vertex_buffer(0, mesh.get_raw_buffer(Mesh::POSITION).unwrap().slice(..));
            render_pass.set_vertex_buffer(1, mesh.get_raw_buffer(Mesh::NORMAL).unwrap().slice(..));
            render_pass.set_vertex_buffer(2, mesh.get_raw_buffer(Mesh::UV).unwrap().slice(..));
            render_pass.set_vertex_buffer(3, mesh.get_raw_buffer(Mesh::TANGENT).unwrap().slice(..));
            render_pass.set_vertex_buffer(4, mesh.get_raw_buffer(Mesh::COLOR).unwrap().slice(..));

            render_pass.set_vertex_buffer(5, self.instances[&id].get_raw().unwrap().slice(..));

            render_pass.draw_indexed(0..mesh.indices().len() as u32, 0, 0..instances.len() as u32);
        }

        Ok(())
    }
}
