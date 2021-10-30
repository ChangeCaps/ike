use bytemuck::bytes_of;

use crate::{
    prelude::*,
    render_texture::RenderTexture,
    renderer::{render_device, render_queue, EdgeSlotInfo, GraphError, NodeEdge},
};

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    min_log_lum: f32,
    inv_log_lum_range: f32,
    time_coefficient: f32,
    num_pixels: u32,
}

pub struct LuminanceBuffer {
    pub buffer: wgpu::Buffer,
}

pub struct LuminanceBufferNode {
    pub min_log_lum: f32,
    pub log_lum_range: f32,
    pub time_coefficient: f32,
    uniform_buffer: Option<wgpu::Buffer>,
    histogram_buffer: Option<wgpu::Buffer>,
    hist_bind_group: Option<wgpu::BindGroup>,
    hist_layout: Option<wgpu::BindGroupLayout>,
    hist_pipeline: Option<wgpu::ComputePipeline>,
    avg_bind_group: Option<wgpu::BindGroup>,
    avg_layout: Option<wgpu::BindGroupLayout>,
    avg_pipeline: Option<wgpu::ComputePipeline>,
}

impl Default for LuminanceBufferNode {
    #[inline]
    fn default() -> Self {
        Self {
            min_log_lum: -10.0,
            log_lum_range: 12.0,
            time_coefficient: 0.7,
            uniform_buffer: None,
            histogram_buffer: None,
            hist_bind_group: None,
            hist_layout: None,
            hist_pipeline: None,
            avg_bind_group: None,
            avg_layout: None,
            avg_pipeline: None,
        }
    }
}

impl LuminanceBufferNode {
    pub const HDR_TEXTURE: &'static str = "hdr_texture";
    pub const LUMINANCE_BUFFER: &'static str = "luminance_buffer";

    fn uniforms(&self, target_size: UVec2) -> Uniforms {
        Uniforms {
            min_log_lum: self.min_log_lum,
            inv_log_lum_range: 1.0 / self.log_lum_range,
            time_coefficient: self.time_coefficient,
            num_pixels: target_size.x * target_size.y,
        }
    }

    fn create_buffers(&mut self, output: &mut NodeEdge, target_size: UVec2) {
        if self.uniform_buffer.is_none() {
            let device = render_device();

            let uniform_buffer = device.create_buffer_init(&wgpu::BufferInitDescriptor {
                label: None,
                contents: bytes_of(&self.uniforms(target_size)),
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
            });

            let histogram_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: None,
                size: std::mem::size_of::<[u32; 256]>() as u64,
                mapped_at_creation: false,
                usage: wgpu::BufferUsages::STORAGE,
            });

            self.uniform_buffer = Some(uniform_buffer);
            self.histogram_buffer = Some(histogram_buffer);

            let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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

            self.hist_layout = Some(layout);

            let buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: None,
                size: 4,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::STORAGE,
                mapped_at_creation: false,
            });

            let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: None,
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

            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: buffer.as_entire_binding(),
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

            self.avg_bind_group = Some(bind_group);
            self.avg_layout = Some(layout);

            output.set(Self::LUMINANCE_BUFFER, LuminanceBuffer { buffer });
        }
    }

    fn texture_recreated(&mut self, texture: &wgpu::TextureView) {
        let device = render_device();

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: self.hist_layout.as_ref().unwrap(),
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

        self.hist_bind_group = Some(bind_group);
    }

    fn create_resources(&mut self) {
        if self.hist_pipeline.is_none() {
            let device = render_device();

            let module =
                device.create_shader_module(&ike_wgpu::include_wgsl!("avg_lum_hist.comp.wgsl"));

            let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[self.hist_layout.as_ref().unwrap()],
                push_constant_ranges: &[],
            });

            let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: None,
                layout: Some(&layout),
                module: &module,
                entry_point: "main",
            });

            self.hist_pipeline = Some(pipeline);

            let module = device.create_shader_module(&ike_wgpu::include_wgsl!("avg_lum.comp.wgsl"));

            let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[self.avg_layout.as_ref().unwrap()],
                push_constant_ranges: &[],
            });

            let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: None,
                layout: Some(&layout),
                module: &module,
                entry_point: "main",
            });

            self.avg_pipeline = Some(pipeline);
        }
    }
}

impl RenderNode for LuminanceBufferNode {
    #[inline]
    fn input(&self) -> Vec<EdgeSlotInfo> {
        vec![EdgeSlotInfo::new::<RenderTexture>(Self::HDR_TEXTURE)]
    }

    #[inline]
    fn output(&self) -> Vec<EdgeSlotInfo> {
        vec![EdgeSlotInfo::new::<LuminanceBuffer>(Self::LUMINANCE_BUFFER)]
    }

    #[inline]
    fn run(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        input: &NodeEdge,
        output: &mut NodeEdge,
    ) -> Result<(), GraphError> {
        let target = input.get::<RenderTexture>(Self::HDR_TEXTURE)?;

        let view = target.texture().create_view(&Default::default());

        self.create_buffers(output, target.size);

        if target.recreate {
            self.texture_recreated(&view);
        }

        self.create_resources();

        if let Some(ref buffer) = self.uniform_buffer {
            render_queue().write_buffer(buffer, 0, bytes_of(&self.uniforms(target.size)));
        }

        let mut compute_pass = encoder.begin_compute_pass(&Default::default());

        compute_pass.set_pipeline(self.hist_pipeline.as_ref().unwrap());
        compute_pass.set_bind_group(0, self.hist_bind_group.as_ref().unwrap(), &[]);
        compute_pass.dispatch(target.size.x / 16, target.size.y / 16, 1);

        compute_pass.set_pipeline(self.avg_pipeline.as_ref().unwrap());
        compute_pass.set_bind_group(0, self.avg_bind_group.as_ref().unwrap(), &[]);
        compute_pass.dispatch(1, 1, 1);

        Ok(())
    }
}
