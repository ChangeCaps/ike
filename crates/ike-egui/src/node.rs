use std::collections::HashMap;

use bytemuck::{Zeroable, bytes_of, cast_slice};
use ike_assets::Assets;
use ike_core::*;
use ike_render::*;

use crate::EguiTextures;

struct ShaderResources {
	shader: wgpu::ShaderModule,
	group: wgpu::BindGroupLayout,
	layout: wgpu::PipelineLayout,
	sampler: wgpu::Sampler,
	pipelines: HashMap<RenderTarget, wgpu::RenderPipeline>,
}

impl Default for ShaderResources {
	#[inline]
	fn default() -> Self {
		Self::new()
	}
}

impl ShaderResources {
	fn new() -> Self {
		let device = render_device();

		let shader = device.create_shader_module(&wgpu::include_wgsl!("egui.wgsl"));

		let group = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
			label: None,
			entries: &[
				wgpu::BindGroupLayoutEntry {
					binding: 0,
					ty: wgpu::BindingType::Buffer {
						ty: wgpu::BufferBindingType::Uniform,
						has_dynamic_offset: false,
						min_binding_size: None,
					},
					visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
					count: None,
				},
				wgpu::BindGroupLayoutEntry {
					binding: 1,
					ty: wgpu::BindingType::Texture {
						sample_type: wgpu::TextureSampleType::Float { filterable: true },
						view_dimension: wgpu::TextureViewDimension::D2,
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

		let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
			label: None,
			bind_group_layouts: &[&group],
			push_constant_ranges: &[],
		});

		let sampler = device.create_sampler(&Default::default());

		Self {
			shader,
			group,
			layout,
			sampler,
			pipelines: HashMap::new(),
		}
	}

	fn create_pipeline(&mut self, target: RenderTarget) {
		let device = render_device();

		let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
			label: None,
			layout: Some(&self.layout),
			vertex: wgpu::VertexState {
				module: &self.shader,	
				entry_point: "main",
				buffers: &[

				],
			},
			fragment: Some(wgpu::FragmentState {
				module: &self.shader,
				entry_point: "main",
				targets: &[wgpu::ColorTargetState {
					format: target.format,
					blend: Some(wgpu::BlendState::ALPHA_BLENDING),
					write_mask: wgpu::ColorWrites::ALL,
				}],
			}),
			primitive: wgpu::PrimitiveState::default(),
			multisample: wgpu::MultisampleState {
				count: target.samples,
				..Default::default()
			},
			depth_stencil: None,
		});

		self.pipelines.insert(target, pipeline);
	}
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
struct Uniforms {
	size: [f32; 2],
}

struct Mesh {
	uniforms: Option<wgpu::Buffer>,
	texture: Option<egui::TextureId>,
	group: Option<wgpu::BindGroup>,
	vertices: Buffer,
	indices: Buffer,
}

impl Default for Mesh {
	fn default() -> Self {
		Self {
			uniforms: None,
			texture: None,
			group: None,
			vertices: Buffer::new(wgpu::BufferUsages::VERTEX),
			indices: Buffer::new(wgpu::BufferUsages::INDEX),
		}	
	}
}

#[derive(Default)]
pub struct EguiNode {
	clear_color: Option<Color>,
	meshes: Vec<Mesh>,
}

impl EguiNode {
	pub const TARGET: &'static str = "target";

	#[inline]
	pub fn clear(color: Color) -> Self {
		Self {
			clear_color: Some(color),
			..Default::default()
		}
	}
}

impl RenderNode for EguiNode {
	fn input(&self) -> Vec<EdgeSlotInfo> {
		vec![
			EdgeSlotInfo::new::<RenderTexture>(Self::TARGET)
		]
	}

	fn update(&mut self, world: &mut World) {
		world.init_resource::<ShaderResources>();
	}

    fn run(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        world: &World,
        input: &NodeInput,
        _output: &mut NodeEdge,
    ) -> Result<(), GraphError> {
		let target = input.get::<RenderTexture>(Self::TARGET)?;

		let mut resources = world.write_resource::<ShaderResources>().unwrap();

		let egui_textures = world.read_resource::<EguiTextures>().unwrap();
		let textures = world.read_resource::<Assets<Texture>>().unwrap();
		let ctx = world.read_resource::<egui::CtxRef>().unwrap();

		let (_output, shapes) = ctx.end_frame();
		let meshes = ctx.tessellate(shapes);

		for (i, egui::ClippedMesh(_, mesh)) in meshes.into_iter().enumerate() {
			if i >= self.meshes.len() {
				self.meshes.push(Default::default());
			}

			let uniforms = Uniforms {
				size: target.size.as_vec2().into(),
			};

			let gpu_mesh = &mut self.meshes[i];

			if let Some(ref buffer) = gpu_mesh.uniforms {
				render_queue().write_buffer(buffer, 0, bytes_of(&uniforms));
			} else {
				let buffer = render_device().create_buffer_init(&wgpu::BufferInitDescriptor {
					label: None,
					contents: bytes_of(&uniforms),
					usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
				});

				gpu_mesh.uniforms = Some(buffer);
			}

			gpu_mesh.vertices.write(cast_slice(&mesh.vertices)); 
			gpu_mesh.indices.write(cast_slice(&mesh.indices));

			gpu_mesh.vertices.raw();
			gpu_mesh.indices.raw();

			if gpu_mesh.texture != Some(mesh.texture_id) {
				gpu_mesh.texture = Some(mesh.texture_id);

				match mesh.texture_id {
					egui::TextureId::Egui => {

					},
					id @ egui::TextureId::User(_) => {
						let handle = egui_textures.get_texture(&id).unwrap();

						let texture = textures.get(handle).unwrap();
						let view = texture.texture().create_view(&Default::default());

						let group = render_device().create_bind_group(&wgpu::BindGroupDescriptor {
							label: None,
							layout: &resources.group,
							entries: &[
								wgpu::BindGroupEntry {
									binding: 0,
									resource: gpu_mesh.uniforms.as_ref().unwrap().as_entire_binding(),
								},
								wgpu::BindGroupEntry {
									binding: 1,
									resource: wgpu::BindingResource::TextureView(&view),
								},
								wgpu::BindGroupEntry {
									binding: 2,
									resource: wgpu::BindingResource::Sampler(&resources.sampler),
								}
							],
						});

						gpu_mesh.group = Some(group);
					}
				}
			}
		}

		if !resources.pipelines.contains_key(&target.target()) {
			resources.create_pipeline(target.target());
		}

		let pipeline = &resources.pipelines[&target.target()];

		let view = target.texture().create_view(&Default::default());

		let load = if let Some(color) = self.clear_color {
			wgpu::LoadOp::Clear(color.into())
		} else {
			wgpu::LoadOp::Load
		};

		let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
			label: Some("egui pass"),
			color_attachments: &[
				wgpu::RenderPassColorAttachment {
					view: &view,
					resolve_target: None,
					ops: wgpu::Operations {
						load,
						store: true,
					}
				}
			],
			depth_stencil_attachment: None,
		});

		render_pass.set_pipeline(pipeline);

		Ok(())
    } 
}
