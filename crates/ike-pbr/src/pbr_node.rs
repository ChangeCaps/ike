use std::collections::HashMap;

use bytemuck::bytes_of;
use ike_assets::{Assets, Handle, HandleId};
use ike_ecs::{FromWorld, World};
use ike_light::LightBindings;
use ike_math::Mat4;
use ike_render::{
    include_wgsl, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
    BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType, Buffer,
    BufferBindingType, BufferInitDescriptor, BufferUsages, ColorTargetState, ColorWrites,
    CompareFunction, DepthStencilState, FragmentState, Image, ImageTexture, IndexFormat, LoadOp,
    Mesh, MeshBindings, MeshBuffers, MultisampleState, Operations, PipelineLayout,
    PipelineLayoutDescriptor, RawCamera, RawColor, RenderContext, RenderDevice, RenderGraphContext,
    RenderNode, RenderPassColorAttachment, RenderPassDepthStencilAttachemnt, RenderPassDescriptor,
    RenderPipeline, RenderPipelineDescriptor, RenderQueue, SamplerBindingType, ShaderStages,
    SlotInfo, TextureFormat, TextureSampleType, TextureView, TextureViewDimension, VertexAttribute,
    VertexBufferLayout, VertexFormat, VertexState, VertexStepMode,
};
use ike_transform::GlobalTransform;

use crate::PbrMaterial;

struct MaterialBinding {
    base_color: Option<Handle<Image>>,
    metallic_roughness: Option<Handle<Image>>,
    normal_map: Option<Handle<Image>>,
    emission: Option<Handle<Image>>,
    buffer: Buffer,
    bind_group: BindGroup,
}

impl MaterialBinding {
    fn get_texture<'a>(
        image: &Option<Handle<Image>>,
        default_image: &'a ImageTexture,
        image_textures: &'a Assets<ImageTexture>,
    ) -> &'a ImageTexture {
        image
            .as_ref()
            .and_then(|image| image_textures.get(image))
            .unwrap_or(default_image)
    }

    pub fn new(
        material: &PbrMaterial,
        device: &RenderDevice,
        layout: &BindGroupLayout,
        default_image: &ImageTexture,
        image_textures: &Assets<ImageTexture>,
    ) -> Self {
        let raw_material = material.as_raw();

        let buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("pbr_material_buffer"),
            contents: bytes_of(&raw_material),
            usage: BufferUsages::COPY_DST | BufferUsages::UNIFORM,
        });

        let base_color =
            Self::get_texture(&material.base_color_texture, default_image, image_textures);
        let metallic_roughness = Self::get_texture(
            &material.metallic_roughness_texture,
            default_image,
            image_textures,
        );
        let emission = Self::get_texture(&material.emission_texture, default_image, image_textures);
        let normal_map = Self::get_texture(&material.normal_map, default_image, image_textures);

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("pbr_material_bind_group"),
            layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: buffer.raw().as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(base_color.texture_view.raw()),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::Sampler(&base_color.sampler),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: BindingResource::TextureView(metallic_roughness.texture_view.raw()),
                },
                BindGroupEntry {
                    binding: 4,
                    resource: BindingResource::Sampler(&metallic_roughness.sampler),
                },
                BindGroupEntry {
                    binding: 5,
                    resource: BindingResource::TextureView(emission.texture_view.raw()),
                },
                BindGroupEntry {
                    binding: 6,
                    resource: BindingResource::Sampler(&emission.sampler),
                },
                BindGroupEntry {
                    binding: 7,
                    resource: BindingResource::TextureView(normal_map.texture_view.raw()),
                },
                BindGroupEntry {
                    binding: 8,
                    resource: BindingResource::Sampler(&normal_map.sampler),
                },
            ],
        });

        Self {
            base_color: material.base_color_texture.clone(),
            metallic_roughness: material.metallic_roughness_texture.clone(),
            normal_map: material.normal_map.clone(),
            emission: material.emission_texture.clone(),
            buffer,
            bind_group,
        }
    }
}

pub struct PbrResources {
    pub object_bind_group_layout: BindGroupLayout,
    pub material_bind_group_layout: BindGroupLayout,
    pub pipeline_layout: PipelineLayout,
    pub render_pipeline: RenderPipeline,
    pub default_image: ImageTexture,
}

impl FromWorld for PbrResources {
    fn from_world(world: &mut World) -> Self {
        let device = world.resource::<RenderDevice>();
        let queue = world.resource::<RenderQueue>();
        let light_bindings = world.resource::<LightBindings>();

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

        let material_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("ike_pbr_material_bind_group_layout"),
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        visibility: ShaderStages::VERTEX_FRAGMENT,
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        ty: BindingType::Texture {
                            sample_type: TextureSampleType::Float { filterable: true },
                            view_dimension: TextureViewDimension::D2,
                            multisampled: false,
                        },
                        visibility: ShaderStages::VERTEX_FRAGMENT,
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 2,
                        ty: BindingType::Sampler(SamplerBindingType::Filtering),
                        visibility: ShaderStages::VERTEX_FRAGMENT,
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 3,
                        ty: BindingType::Texture {
                            sample_type: TextureSampleType::Float { filterable: true },
                            view_dimension: TextureViewDimension::D2,
                            multisampled: false,
                        },
                        visibility: ShaderStages::VERTEX_FRAGMENT,
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 4,
                        ty: BindingType::Sampler(SamplerBindingType::Filtering),
                        visibility: ShaderStages::VERTEX_FRAGMENT,
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 5,
                        ty: BindingType::Texture {
                            sample_type: TextureSampleType::Float { filterable: true },
                            view_dimension: TextureViewDimension::D2,
                            multisampled: false,
                        },
                        visibility: ShaderStages::VERTEX_FRAGMENT,
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 6,
                        ty: BindingType::Sampler(SamplerBindingType::Filtering),
                        visibility: ShaderStages::VERTEX_FRAGMENT,
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 7,
                        ty: BindingType::Texture {
                            sample_type: TextureSampleType::Float { filterable: false },
                            view_dimension: TextureViewDimension::D2,
                            multisampled: false,
                        },
                        visibility: ShaderStages::VERTEX_FRAGMENT,
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 8,
                        ty: BindingType::Sampler(SamplerBindingType::NonFiltering),
                        visibility: ShaderStages::VERTEX_FRAGMENT,
                        count: None,
                    },
                ],
            });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("ike_pbr_pipeline_layout"),
            bind_group_layouts: &[
                &object_bind_group_layout,
                &material_bind_group_layout,
                &light_bindings.bind_group_layout,
            ],
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
                    VertexBufferLayout {
                        step_mode: VertexStepMode::Vertex,
                        array_stride: 8,
                        attributes: &[VertexAttribute {
                            format: VertexFormat::Float32x2,
                            shader_location: 2,
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
            depth_stencil: Some(DepthStencilState {
                format: TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: CompareFunction::LessEqual,
                stencil: Default::default(),
                bias: Default::default(),
            }),
            multisample: MultisampleState {
                count: 4,
                ..Default::default()
            },
            primitive: Default::default(),
            multiview: None,
        });

        let default_image = Image::default().create_texture(&device, &queue);

        Self {
            object_bind_group_layout,
            material_bind_group_layout,
            pipeline_layout,
            render_pipeline,
            default_image,
        }
    }
}

#[derive(Default)]
pub struct PbrNode {
    mesh_bindings: MeshBindings,
    material_bindings: HashMap<HandleId, MaterialBinding>,
}

impl PbrNode {
    pub const RENDER_TARGET: &'static str = "render_target";
    pub const MSAA_TEXTURE: &'static str = "msaa_texture";
    pub const DEPTH: &'static str = "depth";
    pub const CAMERA: &'static str = "camera";
}

impl RenderNode for PbrNode {
    fn input() -> Vec<SlotInfo> {
        vec![
            SlotInfo::new::<TextureView>(Self::RENDER_TARGET),
            SlotInfo::new::<TextureView>(Self::MSAA_TEXTURE),
            SlotInfo::new::<TextureView>(Self::DEPTH),
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
        let msaa_texture = graph_context.get_input::<TextureView>(Self::MSAA_TEXTURE)?;
        let depth = graph_context.get_input::<TextureView>(Self::DEPTH)?;
        let camera = graph_context.get_input::<RawCamera>(Self::CAMERA)?;

        let meshes = world.resource::<Assets<Mesh>>();
        let mesh_buffers = world.resource::<Assets<MeshBuffers>>();
        let materials = world.resource::<Assets<PbrMaterial>>();
        let image_textures = world.resource::<Assets<ImageTexture>>();
        let pbr_resources = world.resource::<PbrResources>();
        let light_bindings = world.resource::<LightBindings>();

        let mesh_query = world
            .query::<(
                &Handle<Mesh>,
                &Handle<PbrMaterial>,
                Option<&GlobalTransform>,
            )>()
            .unwrap();

        for (i, (_, material_handle, global_transform)) in mesh_query.iter().enumerate() {
            self.mesh_bindings.require(i, &render_context.device);

            let transform = global_transform.map_or(Mat4::IDENTITY, |transform| transform.matrix());

            self.mesh_bindings[i].write(&render_context.queue, transform, camera);

            if !self.material_bindings.contains_key(&material_handle.into()) {
                let material = materials.get(material_handle).unwrap();

                let material_binding = MaterialBinding::new(
                    material,
                    &render_context.device,
                    &pbr_resources.material_bind_group_layout,
                    &pbr_resources.default_image,
                    &image_textures,
                );

                self.material_bindings
                    .insert(material_handle.into(), material_binding);
            }
        }

        let mut render_pass = render_context
            .encoder
            .begin_render_pass(&RenderPassDescriptor {
                label: Some("pbr_pass"),
                color_attachments: &[RenderPassColorAttachment {
                    view: msaa_texture.raw(),
                    resolve_target: Some(target.raw()),
                    ops: Operations {
                        load: LoadOp::Clear(RawColor::BLACK),
                        store: true,
                    },
                }],
                depth_stencil_attachment: Some(RenderPassDepthStencilAttachemnt {
                    view: depth.raw(),
                    depth_ops: Some(Operations {
                        load: LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

        render_pass.set_pipeline(&pbr_resources.render_pipeline);

        for (i, (mesh_handle, material_handle, _)) in mesh_query.iter().enumerate() {
            let object_binding = &self.mesh_bindings[i];
            let material_binding = &self.material_bindings[&material_handle.into()];
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

            if let Some(uv_0) = mesh_buffers.get_attribute(Mesh::UV_0) {
                render_pass.set_vertex_buffer(2, uv_0.raw().slice(..));
            } else {
                continue;
            }

            render_pass.set_index_buffer(
                mesh_buffers.get_indices().raw().slice(..),
                IndexFormat::Uint32,
            );

            render_pass.set_bind_group(0, &object_binding.bind_group, &[]);
            render_pass.set_bind_group(1, &material_binding.bind_group, &[]);
            render_pass.set_bind_group(2, &light_bindings.bind_group, &[]);

            render_pass.draw_indexed(0..mesh.get_indices().len() as u32, 0, 0..1);
        }

        Ok(())
    }
}
