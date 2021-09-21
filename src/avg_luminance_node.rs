use bytemuck::bytes_of;

use crate::prelude::*;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
	min_log_lum: f32,
	inv_log_lum_range: f32,
}

#[derive(Default)]
pub struct AvgLuminanceNode {
	uniform_buffer: Option<wgpu::Buffer>,
	histogram_buffer: Option<wgpu::Buffer>,
	bind_group: Option<wgpu::BindGroup>,
	layout: Option<wgpu::BindGroupLayout>,
    pipeline: Option<wgpu::ComputePipeline>,
}

impl AvgLuminanceNode {
	fn uniforms(&self) -> Uniforms {
		Uniforms {
			min_log_lum: 0.1f32.log2(),
			inv_log_lum_range: 1.0 / 800.0f32.log2(),
		}
	}

	fn create_buffers(&mut self, ctx: &RenderCtx) {
		if self.uniform_buffer.is_none() {
			let uniform_buffer = ctx.device.create_buffer_init(&wgpu::BufferInitDescriptor {
				label: None,
				contents: bytes_of(&self.uniforms()),
				usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
			});

			let histogram_buffer = ctx.device.create_buffer(&wgpu::BufferDescriptor {
				label: None,
				size: std::mem::size_of::<[u32; 256]>() as u64,
				mapped_at_creation: false,	
				usage: wgpu::BufferUsages::STORAGE,
			});

			self.uniform_buffer = Some(uniform_buffer);
			self.histogram_buffer = Some(histogram_buffer);

			let layout = ctx.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
				label: None,
				entries: &[
					wgpu::BindGroupLayoutEntry {
						binding: 0,
						ty: wgpu::BindingType::Texture {
							sample_type: wgpu::TextureSampleType::Float { filterable: false },
							view_dimension: wgpu::TextureViewDimension::D2,
							multisampled: false,
						},
						visibility: wgpu::ShaderStages::COMPUTE,
						count: None,
					},
					wgpu::BindGroupLayoutEntry {
						binding: 1,
						ty: wgpu::BindingType::Buffer {
							ty: wgpu::BufferBindingType::Storage { read_only: false },
							has_dynamic_offset: false,
							min_binding_size: None,
						},
						visibility: wgpu::ShaderStages::COMPUTE,
						count: None,
					},
					wgpu::BindGroupLayoutEntry {
						binding: 2,
						ty: wgpu::BindingType::Buffer {
							ty: wgpu::BufferBindingType::Uniform,
							has_dynamic_offset: false,
							min_binding_size: None,
						},
						visibility: wgpu::ShaderStages::COMPUTE,
						count: None,
					},
				],
			});

			self.layout = Some(layout);
		}
	}

	fn texture_recreated(&mut self, ctx: &RenderCtx, texture: &wgpu::TextureView) {
		let bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
			label: None,
			layout: self.layout.as_ref().unwrap(),
			entries: &[
				wgpu::BindGroupEntry {
					binding: 0,
					resource: wgpu::BindingResource::TextureView(texture),	
				},
				wgpu::BindGroupEntry {
					binding: 1,
					resource: self.histogram_buffer.as_ref().unwrap().as_entire_binding(),	
				},
				wgpu::BindGroupEntry {
					binding: 2,
					resource: self.uniform_buffer.as_ref().unwrap().as_entire_binding(),	
				},
			],
		});

		self.bind_group = Some(bind_group);
	}

    fn create_resources(&mut self, ctx: &RenderCtx) {
        if self.pipeline.is_none() {
            let module = ctx
                .device
                .create_shader_module(&ike_wgpu::include_wgsl!("avg_lum.comp.wgsl"));

			let layout = ctx
				.device
				.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
					label: None,
					bind_group_layouts: &[self.layout.as_ref().unwrap()],
					push_constant_ranges: &[],
				});

			let pipeline = ctx.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
				label: None,
				layout: Some(&layout),
				module: &module,
				entry_point: "main",
			});

			self.pipeline = Some(pipeline);
        }
    } 
}

impl<S> RenderNode<S> for AvgLuminanceNode {
    #[inline]
    fn run(&mut self, ctx: &mut RenderNodeCtx) {
		let target = if let Some(target) = ctx.data.get::<HdrTarget>() {
			target
		} else {
			return;
		};

		let target_size = ctx.data.get::<TargetSize>().cloned().unwrap_or(TargetSize(ctx.view.size()));

		self.create_buffers(ctx.render_ctx);

		if target.recreated {
			self.texture_recreated(ctx.render_ctx, &target.view);
		}

		self.create_resources(ctx.render_ctx);

		let mut compute_pass = ctx.encoder.begin_compute_pass(&Default::default());

		compute_pass.set_pipeline(self.pipeline.as_ref().unwrap());

		compute_pass.set_bind_group(0, self.bind_group.as_ref().unwrap(), &[]);

		compute_pass.dispatch(target_size.0.x / 16, target_size.0.y / 16, 1);
	}
}
