use std::collections::HashMap;

use bytemuck::bytes_of;
use glam::Mat4;
use ike_assets::{Assets, Handle};
use ike_render::*;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    view_to_world: Mat4,
    clip_to_view: Mat4,
}

#[derive(Default)]
pub struct SkyNode {
    current_env: Option<Handle<Environment>>,
    uniform_buffer: Option<wgpu::Buffer>,
    bind_group_layout: Option<wgpu::BindGroupLayout>,
    bind_group: Option<wgpu::BindGroup>,
    pipelines: HashMap<RenderTarget, wgpu::RenderPipeline>,
}

impl SkyNode {
    pub const TARGET: &'static str = "target";
    pub const CAMERA: &'static str = "camera";

    #[inline]
    fn create_resources(&mut self) {
        if self.bind_group.is_none() {
            let device = render_device();

            let buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: None,
                size: std::mem::size_of::<Uniforms>() as u64,
                mapped_at_creation: false,
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
            });

            let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            min_binding_size: None,
                            has_dynamic_offset: false,
                        },
                        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: false },
                            view_dimension: wgpu::TextureViewDimension::Cube,
                            multisampled: false,
                        },
                        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        ty: wgpu::BindingType::Sampler {
                            filtering: true,
                            comparison: false,
                        },
                        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                        count: None,
                    },
                ],
            });

            self.uniform_buffer = Some(buffer);
            self.bind_group_layout = Some(layout);
        }
    }

    #[inline]
    fn set_texture(&mut self, texture: &CubeTexture) {
        if self.bind_group_layout.is_none() {
            self.create_resources();
        }

        let bind_group = render_device().create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &self.bind_group_layout.as_ref().unwrap(),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.uniform_buffer.as_ref().unwrap().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&texture.view()),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&render_device().create_sampler(
                        &wgpu::SamplerDescriptor {
                            min_filter: wgpu::FilterMode::Linear,
                            mag_filter: wgpu::FilterMode::Linear,
                            ..Default::default()
                        },
                    )),
                },
            ],
        });

        self.bind_group = Some(bind_group);
    }

    #[inline]
    fn create_pipeline(&self, target: RenderTarget) -> wgpu::RenderPipeline {
        let device = render_device();

        let module = device.create_shader_module(&wgpu::include_wgsl!("sky.wgsl"));

        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[self.bind_group_layout.as_ref().unwrap()],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &module,
                entry_point: "main",
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &module,
                entry_point: "main",
                targets: &[wgpu::ColorTargetState {
                    format: target.format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                }],
            }),
            primitive: Default::default(),
            multisample: wgpu::MultisampleState {
                count: target.samples,
                ..Default::default()
            },
            depth_stencil: None,
        });

        pipeline
    }
}

impl RenderNode for SkyNode {
    fn input(&self) -> Vec<EdgeSlotInfo> {
        vec![
            EdgeSlotInfo::new::<RenderTexture>(Self::TARGET),
            EdgeSlotInfo::new::<Camera>(Self::CAMERA),
        ]
    }

    fn run(
        &mut self,
        encoder: &mut ike_render::wgpu::CommandEncoder,
        world: &ike_core::WorldRef,
        input: &NodeInput,
        _output: &mut NodeEdge,
    ) -> Result<(), GraphError> {
        let target = input.get::<RenderTexture>(Self::TARGET).unwrap();
        let camera = input.get::<Camera>(Self::CAMERA).unwrap();

        let view = target.texture().create_view(&Default::default());

        self.create_resources();

        let uniforms = Uniforms {
            view_to_world: camera.view,
            clip_to_view: camera.proj.inverse(),
        };

        render_queue().write_buffer(
            self.uniform_buffer.as_ref().unwrap(),
            0,
            bytes_of(&uniforms),
        );

        if !self.pipelines.contains_key(&target.target()) {
            let pipeline = self.create_pipeline(target.target());

            self.pipelines.insert(target.target(), pipeline);
        }

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("sky pass"),
            color_attachments: &[wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: true,
                },
            }],
            depth_stencil_attachment: None,
        });

        if let Some(env) = world.get_resource::<Handle<Environment>>() {
            if self.current_env.as_ref() != Some(&env) {
                self.current_env = Some(env.clone());

                let envs = world.get_resource::<Assets<Environment>>().unwrap();

                let env = envs.get(&env).unwrap();

                self.set_texture(&env.env_texture);
            }

            let pipeline = &self.pipelines[&target.target()];

            render_pass.set_pipeline(pipeline);

            render_pass.set_bind_group(0, self.bind_group.as_ref().unwrap(), &[]);

            render_pass.draw(0..3, 0..1);
        }

        Ok(())
    }
}
