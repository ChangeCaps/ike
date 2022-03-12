use std::{collections::HashMap, num::NonZeroU32};

use ike_assets::{Assets, Handle, HandleId};
use ike_ecs::{FromWorld, World};
use ike_math::Mat4;
use ike_render::{
    include_wgsl, CompareFunction, DepthStencilState, IndexFormat, LoadOp, Mesh, MeshBinding,
    MeshBuffers, Operations, PipelineLayoutDescriptor, RawCamera, RenderContext, RenderDevice,
    RenderGraphContext, RenderGraphResult, RenderNode, RenderPassDepthStencilAttachment,
    RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor, RenderQueue, TextureFormat,
    TextureViewDescriptor, VertexAttribute, VertexBufferLayout, VertexFormat, VertexState,
    VertexStepMode,
};
use ike_transform::{GlobalTransform, Transform};

use crate::{AmbientLight, DirectionalLight, LightBindings, RawLights, MAX_DIRECTIONAL_LIGHTS};

pub struct LightPipeline {
    pub render_pipeline: RenderPipeline,
}

impl FromWorld for LightPipeline {
    fn from_world(world: &mut World) -> Self {
        let device = world.resource::<RenderDevice>();

        let bind_group_layout = MeshBinding::bind_group_layout(&device);

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("ike_light_pipeline_layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let module = &device.create_shader_module(&include_wgsl!("depth.wgsl"));

        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("ike_light_render_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module,
                entry_point: "vert",
                buffers: &[
                    VertexBufferLayout {
                        array_stride: 12,
                        step_mode: VertexStepMode::Vertex,
                        attributes: &[VertexAttribute {
                            format: VertexFormat::Float32x3,
                            shader_location: 0,
                            offset: 0,
                        }],
                    },
                    VertexBufferLayout {
                        array_stride: 64,
                        step_mode: VertexStepMode::Instance,
                        attributes: &[
                            VertexAttribute {
                                format: VertexFormat::Float32x4,
                                shader_location: 1,
                                offset: 0,
                            },
                            VertexAttribute {
                                format: VertexFormat::Float32x4,
                                shader_location: 2,
                                offset: 16,
                            },
                            VertexAttribute {
                                format: VertexFormat::Float32x4,
                                shader_location: 3,
                                offset: 32,
                            },
                            VertexAttribute {
                                format: VertexFormat::Float32x4,
                                shader_location: 4,
                                offset: 48,
                            },
                        ],
                    },
                ],
            },
            fragment: None,
            depth_stencil: Some(DepthStencilState {
                format: TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: CompareFunction::LessEqual,
                stencil: Default::default(),
                bias: Default::default(),
            }),
            primitive: Default::default(),
            multisample: Default::default(),
            multiview: None,
        });

        Self { render_pipeline }
    }
}

#[derive(Default)]
pub struct LightNode {
    directional_light_mesh_bindings: HashMap<HandleId, MeshBinding>,
}

impl RenderNode for LightNode {
    fn update(&mut self, world: &mut World) {
        let queue = world.resource::<RenderQueue>();
        let light_bindings = world.resource::<LightBindings>();
        let ambient_light = world.resource::<AmbientLight>();

        let directional_light_query = world
            .query::<(&DirectionalLight, Option<&GlobalTransform>)>()
            .unwrap();

        let mut raw_lights = RawLights::new();
        raw_lights.ambient = (ambient_light.color * ambient_light.intensity).into();

        for (light, transform) in directional_light_query.iter() {
            let transform = transform.map_or(Mat4::IDENTITY, GlobalTransform::matrix);

            raw_lights.push_directional_light(light.as_raw(transform));
        }

        light_bindings.write(&queue, &raw_lights);
    }

    fn run(
        &mut self,
        _graph_context: &mut RenderGraphContext<'_>,
        render_context: &mut RenderContext,
        world: &World,
    ) -> RenderGraphResult<()> {
        let pipeline = world.resource::<LightPipeline>();
        let light_bindings = world.resource::<LightBindings>();
        let meshes = world.resource::<Assets<Mesh>>();
        let mesh_buffers = world.resource::<Assets<MeshBuffers>>();

        let directional_lights = world
            .query::<(&DirectionalLight, Option<&GlobalTransform>)>()
            .unwrap()
            .iter()
            .take(MAX_DIRECTIONAL_LIGHTS as usize)
            .map(|(light, transform)| {
                let transform = transform.map_or(Transform::IDENTITY, GlobalTransform::transform);

                RawCamera {
                    view: light.view_matrix(),
                    proj: light.projection.matrix(),
                    position: transform.translation,
                }
            })
            .collect::<Vec<_>>();

        let mesh_query = world
            .query::<(&Handle<Mesh>, Option<&GlobalTransform>)>()
            .unwrap();

        for binding in self.directional_light_mesh_bindings.values_mut() {
            binding.clear();
        }

        for camera in directional_lights.iter() {
            for (mesh_handle, transform) in mesh_query.iter() {
                let binding = self
                    .directional_light_mesh_bindings
                    .entry(mesh_handle.into())
                    .or_insert_with(|| MeshBinding::new(&render_context.device));

                let transform = transform.map_or(Mat4::IDENTITY, |transform| transform.matrix());

                binding.push_instance(transform);
            }

            for binding in self.directional_light_mesh_bindings.values_mut() {
                binding.write(&render_context.device, &render_context.queue, camera);
            }
        }

        for (i, _) in directional_lights.iter().enumerate() {
            let view =
                light_bindings
                    .directional_light_shadow_maps
                    .create_view(&TextureViewDescriptor {
                        base_array_layer: i as u32,
                        array_layer_count: NonZeroU32::new(1),
                        ..Default::default()
                    });

            let mut render_pass = render_context
                .encoder
                .begin_render_pass(&RenderPassDescriptor {
                    label: Some("ike_light_pass"),
                    color_attachments: &[],
                    depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                        view: view.raw(),
                        depth_ops: Some(Operations {
                            load: LoadOp::Clear(1.0),
                            store: true,
                        }),
                        stencil_ops: None,
                    }),
                });

            render_pass.set_pipeline(&pipeline.render_pipeline);

            for (handle_id, mesh_binding) in self.directional_light_mesh_bindings.iter() {
                let mesh = meshes.get(*handle_id).unwrap();

                let mesh_buffers = if let Some(mesh_buffers) = mesh_buffers.get(*handle_id) {
                    mesh_buffers
                } else {
                    continue;
                };

                if let Some(position) = mesh_buffers.get_attribute(Mesh::POSITION) {
                    render_pass.set_vertex_buffer(0, position.raw().slice(..));
                } else {
                    continue;
                };

                render_pass.set_index_buffer(
                    mesh_buffers.get_indices().raw().slice(..),
                    IndexFormat::Uint32,
                );

                render_pass.set_vertex_buffer(1, mesh_binding.instance_buffer.raw().slice(..));

                render_pass.set_bind_group(0, &mesh_binding.bind_group, &[]);

                render_pass.draw_indexed(
                    0..mesh.get_indices().len() as u32,
                    0,
                    0..mesh_binding.instances(),
                );
            }
        }

        Ok(())
    }
}
