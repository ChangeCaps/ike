use std::collections::HashMap;

use ike_assets::{Assets, Handle, HandleId};
use ike_ecs::{FromWorld, World};
use ike_light::LightBindings;
use ike_math::Mat4;
use ike_render::{
    include_wgsl, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType,
    BufferBindingType, ColorTargetState, ColorWrites, CompareFunction, DepthStencilState,
    FragmentState, Image, ImageTexture, IndexFormat, LoadOp, Mesh, MeshBinding, MeshBuffers,
    MultisampleState, Operations, PipelineLayout, PipelineLayoutDescriptor, RawCamera, RawColor,
    RenderContext, RenderDevice, RenderGraphContext, RenderNode, RenderPassColorAttachment,
    RenderPassDepthStencilAttachemnt, RenderPassDescriptor, RenderPipeline,
    RenderPipelineDescriptor, RenderQueue, SamplerBindingType, ShaderStages, SlotInfo,
    TextureFormat, TextureSampleType, TextureView, TextureViewDimension, VertexAttribute,
    VertexBufferLayout, VertexFormat, VertexState, VertexStepMode,
};
use ike_transform::GlobalTransform;

use crate::{MaterialBinding, PbrMaterial};

pub struct PbrResources {
    pub object_bind_group_layout: BindGroupLayout,
    pub material_bind_group_layout: BindGroupLayout,
    pub pipeline_layout: PipelineLayout,
    pub render_pipeline: RenderPipeline,
    pub default_normal_map: ImageTexture,
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
                            sample_type: TextureSampleType::Float { filterable: true },
                            view_dimension: TextureViewDimension::D2,
                            multisampled: false,
                        },
                        visibility: ShaderStages::VERTEX_FRAGMENT,
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 8,
                        ty: BindingType::Sampler(SamplerBindingType::Filtering),
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
                        array_stride: 12,
                        attributes: &[VertexAttribute {
                            format: VertexFormat::Float32x3,
                            shader_location: 2,
                            offset: 0,
                        }],
                    },
                    VertexBufferLayout {
                        step_mode: VertexStepMode::Vertex,
                        array_stride: 8,
                        attributes: &[VertexAttribute {
                            format: VertexFormat::Float32x2,
                            shader_location: 3,
                            offset: 0,
                        }],
                    },
                    VertexBufferLayout {
                        step_mode: VertexStepMode::Instance,
                        array_stride: 64,
                        attributes: &[
                            VertexAttribute {
                                format: VertexFormat::Float32x4,
                                shader_location: 4,
                                offset: 0,
                            },
                            VertexAttribute {
                                format: VertexFormat::Float32x4,
                                shader_location: 5,
                                offset: 16,
                            },
                            VertexAttribute {
                                format: VertexFormat::Float32x4,
                                shader_location: 6,
                                offset: 32,
                            },
                            VertexAttribute {
                                format: VertexFormat::Float32x4,
                                shader_location: 7,
                                offset: 48,
                            },
                        ],
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
        let default_normal_map =
            Image::new_2d_with_format(vec![127, 127, 255, 0], 1, 1, TextureFormat::Rgba8Unorm)
                .create_texture(&device, &queue);

        Self {
            object_bind_group_layout,
            material_bind_group_layout,
            pipeline_layout,
            render_pipeline,
            default_normal_map,
            default_image,
        }
    }
}

#[derive(Default)]
pub struct PbrNode {
    mesh_bindings: HashMap<(HandleId, HandleId), MeshBinding>,
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
        let material_bindings = world.resource::<Assets<MaterialBinding>>();
        let pbr_resources = world.resource::<PbrResources>();
        let light_bindings = world.resource::<LightBindings>();

        let mesh_query = world
            .query::<(
                &Handle<Mesh>,
                &Handle<PbrMaterial>,
                Option<&GlobalTransform>,
            )>()
            .unwrap();

        for mesh_binding in self.mesh_bindings.values_mut() {
            mesh_binding.clear();
        }

        for (mesh_handle, material_handle, global_transform) in mesh_query.iter() {
            let mesh_binding = self
                .mesh_bindings
                .entry((mesh_handle.into(), material_handle.into()))
                .or_insert_with(|| MeshBinding::new(&render_context.device));

            let transform = global_transform.map_or(Mat4::IDENTITY, |transform| transform.matrix());

            mesh_binding.push_instance(transform);
        }

        for mesh_binding in self.mesh_bindings.values_mut() {
            mesh_binding.write(&render_context.device, &render_context.queue, camera);
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

        for ((mesh_handle, material_handle), mesh_binding) in self.mesh_bindings.iter() {
            let material_binding =
                if let Some(material_binding) = material_bindings.get(*material_handle) {
                    material_binding
                } else {
                    continue;
                };

            let mesh = meshes.get(*mesh_handle).unwrap();
            let mesh_buffers = mesh_buffers.get(*mesh_handle).unwrap();

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

            if let Some(tangent) = mesh_buffers.get_attribute(Mesh::TANGENT) {
                render_pass.set_vertex_buffer(2, tangent.raw().slice(..));
            } else {
                continue;
            }

            if let Some(uv_0) = mesh_buffers.get_attribute(Mesh::UV_0) {
                render_pass.set_vertex_buffer(3, uv_0.raw().slice(..));
            } else {
                continue;
            }

            render_pass.set_index_buffer(
                mesh_buffers.get_indices().raw().slice(..),
                IndexFormat::Uint32,
            );

            render_pass.set_vertex_buffer(4, mesh_binding.instance_buffer.raw().slice(..));

            render_pass.set_bind_group(0, &mesh_binding.bind_group, &[]);
            render_pass.set_bind_group(1, &material_binding.bind_group, &[]);
            render_pass.set_bind_group(2, &light_bindings.bind_group, &[]);

            render_pass.draw_indexed(
                0..mesh.get_indices().len() as u32,
                0,
                0..mesh_binding.instances(),
            );
        }

        Ok(())
    }
}
