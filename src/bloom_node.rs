use std::num::NonZeroU32;

use bytemuck::bytes_of;
use glam::UVec2;

use crate::prelude::*;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    pre_filter: i32,
    threshold: f32,
    knee: f32,
}

pub struct BloomNode {
    pub threshold: f32,
    pub knee: f32,
    size: UVec2,
    mips: u32,
    tex_down_layout: Option<wgpu::BindGroupLayout>,
    tex_down_bind_groups: Option<Vec<wgpu::BindGroup>>,
    tex_up_layout: Option<wgpu::BindGroupLayout>,
    tex_up_bind_groups: Option<Vec<wgpu::BindGroup>>,
    pre_filter_buffer: Option<wgpu::Buffer>,
    pre_filter_bind_group: Option<wgpu::BindGroup>,
    uniform_buffer: Option<wgpu::Buffer>,
    uniform_bind_group: Option<wgpu::BindGroup>,
    tex_0: Option<wgpu::Texture>,
    tex_1: Option<wgpu::Texture>,
    tex_2: Option<wgpu::Texture>,
    down_pipeline: Option<wgpu::ComputePipeline>,
    up_pipeline: Option<wgpu::ComputePipeline>,
}

impl Default for BloomNode {
    #[inline]
    fn default() -> Self {
        Self {
            threshold: 1.0,
            knee: 0.1,
            size: Default::default(),
            mips: Default::default(),
            tex_down_layout: Default::default(),
            tex_down_bind_groups: Default::default(),
            tex_up_layout: Default::default(),
            tex_up_bind_groups: Default::default(),
            pre_filter_buffer: Default::default(),
            pre_filter_bind_group: Default::default(),
            uniform_buffer: Default::default(),
            uniform_bind_group: Default::default(),
            tex_0: Default::default(),
            tex_1: Default::default(),
            tex_2: Default::default(),
            down_pipeline: Default::default(),
            up_pipeline: Default::default(),
        }
    }
}

impl BloomNode {
    fn uniforms(&self, pre_filter: bool) -> Uniforms {
        Uniforms {
            pre_filter: pre_filter as i32,
            threshold: self.threshold,
            knee: self.knee,
        }
    }

    fn create_texture(&mut self, ctx: &RenderCtx, target_size: UVec2) {
        if self.size != target_size {
            self.size = target_size;

            self.mips = 8;

            self.tex_0 = Some(ctx.device.create_texture(&ike_wgpu::TextureDescriptor {
                label: None,
                size: ike_wgpu::Extent3d {
                    width: target_size.x,
                    height: target_size.y,
                    depth_or_array_layers: 1,
                },
                mip_level_count: self.mips,
                sample_count: 1,
                dimension: ike_wgpu::TextureDimension::D2,
                format: ike_wgpu::TextureFormat::Rgba32Float,
                usage: ike_wgpu::TextureUsages::COPY_DST
                    | ike_wgpu::TextureUsages::STORAGE_BINDING
                    | ike_wgpu::TextureUsages::TEXTURE_BINDING,
            }));

            self.tex_1 = Some(ctx.device.create_texture(&ike_wgpu::TextureDescriptor {
                label: None,
                size: ike_wgpu::Extent3d {
                    width: target_size.x,
                    height: target_size.y,
                    depth_or_array_layers: 1,
                },
                mip_level_count: self.mips,
                sample_count: 1,
                dimension: ike_wgpu::TextureDimension::D2,
                format: ike_wgpu::TextureFormat::Rgba32Float,
                usage: ike_wgpu::TextureUsages::COPY_SRC
                    | ike_wgpu::TextureUsages::COPY_DST
                    | ike_wgpu::TextureUsages::STORAGE_BINDING
                    | ike_wgpu::TextureUsages::TEXTURE_BINDING,
            }));

            self.tex_2 = Some(ctx.device.create_texture(&ike_wgpu::TextureDescriptor {
                label: None,
                size: ike_wgpu::Extent3d {
                    width: target_size.x,
                    height: target_size.y,
                    depth_or_array_layers: 1,
                },
                mip_level_count: self.mips,
                sample_count: 1,
                dimension: ike_wgpu::TextureDimension::D2,
                format: ike_wgpu::TextureFormat::Rgba32Float,
                usage: ike_wgpu::TextureUsages::COPY_SRC
                    | ike_wgpu::TextureUsages::STORAGE_BINDING
                    | ike_wgpu::TextureUsages::TEXTURE_BINDING,
            }));

            let tex_down_layout = self.tex_down_layout.get_or_insert_with(|| {
                let tex_layout =
                    ctx.device
                        .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                            label: None,
                            entries: &[
                                wgpu::BindGroupLayoutEntry {
                                    binding: 0,
                                    ty: wgpu::BindingType::Texture {
                                        sample_type: wgpu::TextureSampleType::Float {
                                            filterable: false,
                                        },
                                        view_dimension: wgpu::TextureViewDimension::D2,
                                        multisampled: false,
                                    },
                                    visibility: wgpu::ShaderStages::COMPUTE,
                                    count: None,
                                },
                                wgpu::BindGroupLayoutEntry {
                                    binding: 1,
                                    ty: wgpu::BindingType::StorageTexture {
                                        access: wgpu::StorageTextureAccess::WriteOnly,
                                        format: wgpu::TextureFormat::Rgba32Float,
                                        view_dimension: wgpu::TextureViewDimension::D2,
                                    },
                                    visibility: wgpu::ShaderStages::COMPUTE,
                                    count: None,
                                },
                            ],
                        });

                tex_layout
            });

            let tex_up_layout = self.tex_up_layout.get_or_insert_with(|| {
                let tex_layout =
                    ctx.device
                        .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                            label: None,
                            entries: &[
                                wgpu::BindGroupLayoutEntry {
                                    binding: 0,
                                    ty: wgpu::BindingType::Texture {
                                        sample_type: wgpu::TextureSampleType::Float {
                                            filterable: false,
                                        },
                                        view_dimension: wgpu::TextureViewDimension::D2,
                                        multisampled: false,
                                    },
                                    visibility: wgpu::ShaderStages::COMPUTE,
                                    count: None,
                                },
                                wgpu::BindGroupLayoutEntry {
                                    binding: 1,
                                    ty: wgpu::BindingType::Texture {
                                        sample_type: wgpu::TextureSampleType::Float {
                                            filterable: false,
                                        },
                                        view_dimension: wgpu::TextureViewDimension::D2,
                                        multisampled: false,
                                    },
                                    visibility: wgpu::ShaderStages::COMPUTE,
                                    count: None,
                                },
                                wgpu::BindGroupLayoutEntry {
                                    binding: 2,
                                    ty: wgpu::BindingType::StorageTexture {
                                        access: wgpu::StorageTextureAccess::WriteOnly,
                                        format: wgpu::TextureFormat::Rgba32Float,
                                        view_dimension: wgpu::TextureViewDimension::D2,
                                    },
                                    visibility: wgpu::ShaderStages::COMPUTE,
                                    count: None,
                                },
                            ],
                        });

                tex_layout
            });

            let mut tex_groups = Vec::new();

            for mip in 0..self.mips - 1 {
                let org = self
                    .tex_0
                    .as_ref()
                    .unwrap()
                    .create_view(&wgpu::TextureViewDescriptor {
                        base_mip_level: mip,
                        mip_level_count: Some(unsafe { NonZeroU32::new_unchecked(1) }),
                        ..Default::default()
                    });

                let target =
                    self.tex_1
                        .as_ref()
                        .unwrap()
                        .create_view(&wgpu::TextureViewDescriptor {
                            base_mip_level: mip + 1,
                            mip_level_count: Some(unsafe { NonZeroU32::new_unchecked(1) }),
                            ..Default::default()
                        });

                let tex_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: None,
                    layout: &tex_down_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(&org),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::TextureView(&target),
                        },
                    ],
                });

                tex_groups.push(tex_group);
            }

            self.tex_down_bind_groups = Some(tex_groups);

            let mut tex_groups = Vec::new();

            for mip in 0..self.mips - 1 {
                let org = self
                    .tex_0
                    .as_ref()
                    .unwrap()
                    .create_view(&wgpu::TextureViewDescriptor {
                        base_mip_level: mip,
                        mip_level_count: Some(unsafe { NonZeroU32::new_unchecked(1) }),
                        ..Default::default()
                    });

                let up = self
                    .tex_1
                    .as_ref()
                    .unwrap()
                    .create_view(&wgpu::TextureViewDescriptor {
                        base_mip_level: mip + 1,
                        mip_level_count: Some(unsafe { NonZeroU32::new_unchecked(1) }),
                        ..Default::default()
                    });

                let target =
                    self.tex_2
                        .as_ref()
                        .unwrap()
                        .create_view(&wgpu::TextureViewDescriptor {
                            base_mip_level: mip,
                            mip_level_count: Some(unsafe { NonZeroU32::new_unchecked(1) }),
                            ..Default::default()
                        });

                let tex_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: None,
                    layout: &tex_up_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(&org),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::TextureView(&up),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: wgpu::BindingResource::TextureView(&target),
                        },
                    ],
                });

                tex_groups.push(tex_group);
            }

            self.tex_up_bind_groups = Some(tex_groups);
        }
    }

    fn create_resources(&mut self, ctx: &RenderCtx) {
        if self.uniform_bind_group.is_none() {
            let uniforms_layout =
                ctx.device
                    .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                        label: None,
                        entries: &[wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            visibility: wgpu::ShaderStages::COMPUTE,
                            count: None,
                        }],
                    });

            let buffer = ctx.device.create_buffer_init(&wgpu::BufferInitDescriptor {
                label: None,
                contents: bytes_of(&self.uniforms(true)),
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
            });

            let bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &uniforms_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                }],
            });

            self.pre_filter_buffer = Some(buffer);
            self.pre_filter_bind_group = Some(bind_group);

            let buffer = ctx.device.create_buffer_init(&wgpu::BufferInitDescriptor {
                label: None,
                contents: bytes_of(&self.uniforms(false)),
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
            });

            let bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &uniforms_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                }],
            });

            self.uniform_buffer = Some(buffer);
            self.uniform_bind_group = Some(bind_group);

            let layout = ctx
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: None,
                    bind_group_layouts: &[self.tex_down_layout.as_ref().unwrap(), &uniforms_layout],
                    push_constant_ranges: &[],
                });

            let module = ctx
                .device
                .create_shader_module(&ike_wgpu::include_wgsl!("bloom_down.comp.wgsl"));

            let down_pipeline =
                ctx.device
                    .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                        label: None,
                        layout: Some(&layout),
                        module: &module,
                        entry_point: "main",
                    });

            self.down_pipeline = Some(down_pipeline);

            let layout = ctx
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: None,
                    bind_group_layouts: &[self.tex_up_layout.as_ref().unwrap()],
                    push_constant_ranges: &[],
                });

            let module = ctx
                .device
                .create_shader_module(&ike_wgpu::include_wgsl!("bloom_up.comp.wgsl"));

            let up_pipeline =
                ctx.device
                    .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                        label: None,
                        layout: Some(&layout),
                        module: &module,
                        entry_point: "main",
                    });

            self.up_pipeline = Some(up_pipeline);
        }
    }
}

impl<S> RenderNode<S> for BloomNode {
    #[inline]
    fn run(&mut self, ctx: &mut RenderNodeCtx) {
        let target = if let Some(target) = ctx.data.get::<HdrTarget>() {
            target
        } else {
            return;
        };

        let target_size = ctx
            .data
            .get::<TargetSize>()
            .cloned()
            .unwrap_or(TargetSize(ctx.view.size()));

        self.create_texture(ctx.render_ctx, target_size.0);
        self.create_resources(ctx.render_ctx);

        ctx.encoder.copy_texture_to_texture(
            wgpu::ImageCopyTexture {
                texture: &target.texture,
                mip_level: 0,
                origin: Default::default(),
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::ImageCopyTexture {
                texture: self.tex_0.as_ref().unwrap(),
                mip_level: 0,
                origin: Default::default(),
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::Extent3d {
                width: self.size.x,
                height: self.size.y,
                depth_or_array_layers: 1,
            },
        );

        ctx.render_ctx.queue.write_buffer(
            self.pre_filter_buffer.as_ref().unwrap(),
            0,
            bytes_of(&self.uniforms(true)),
        );
        ctx.render_ctx.queue.write_buffer(
            self.uniform_buffer.as_ref().unwrap(),
            0,
            bytes_of(&self.uniforms(false)),
        );

        let tex_groups = self.tex_down_bind_groups.as_ref().unwrap();

        for mip in 0..self.mips - 1 {
            let mut compute_pass = ctx.encoder.begin_compute_pass(&Default::default());

            compute_pass.set_pipeline(self.down_pipeline.as_ref().unwrap());

            compute_pass.set_bind_group(0, &tex_groups[mip as usize], &[]);

            if mip == 0 {
                compute_pass.set_bind_group(1, self.pre_filter_bind_group.as_ref().unwrap(), &[]);
            } else {
                compute_pass.set_bind_group(1, self.uniform_bind_group.as_ref().unwrap(), &[]);
            }

            let scale = 2u32.pow(mip as u32 + 1);

            compute_pass.dispatch(self.size.x / 8 / scale + 1, self.size.y / 8 / scale + 1, 1);

            drop(compute_pass);

            if mip == self.mips - 2 {
                break;
            }

            ctx.encoder.copy_texture_to_texture(
                wgpu::ImageCopyTexture {
                    texture: self.tex_1.as_ref().unwrap(),
                    mip_level: mip as u32 + 1,
                    origin: Default::default(),
                    aspect: wgpu::TextureAspect::All,
                },
                wgpu::ImageCopyTexture {
                    texture: self.tex_0.as_ref().unwrap(),
                    mip_level: mip as u32 + 1,
                    origin: Default::default(),
                    aspect: wgpu::TextureAspect::All,
                },
                wgpu::Extent3d {
                    width: self.size.x / scale,
                    height: self.size.y / scale,
                    depth_or_array_layers: 1,
                },
            );
        }

        let tex_groups = self.tex_up_bind_groups.as_ref().unwrap();

        for mip in (0..self.mips - 1).rev() {
            let mut compute_pass = ctx.encoder.begin_compute_pass(&Default::default());

            compute_pass.set_pipeline(self.up_pipeline.as_ref().unwrap());

            compute_pass.set_bind_group(0, &tex_groups[mip as usize], &[]);

            let scale = 2u32.pow(mip as u32);

            compute_pass.dispatch(self.size.x / 8 / scale + 1, self.size.y / 8 / scale + 1, 1);

            drop(compute_pass);

            if mip == 0 {
                break;
            }

            ctx.encoder.copy_texture_to_texture(
                wgpu::ImageCopyTexture {
                    texture: self.tex_2.as_ref().unwrap(),
                    mip_level: mip as u32,
                    origin: Default::default(),
                    aspect: wgpu::TextureAspect::All,
                },
                wgpu::ImageCopyTexture {
                    texture: self.tex_1.as_ref().unwrap(),
                    mip_level: mip as u32,
                    origin: Default::default(),
                    aspect: wgpu::TextureAspect::All,
                },
                wgpu::Extent3d {
                    width: self.size.x / scale,
                    height: self.size.y / scale,
                    depth_or_array_layers: 1,
                },
            );
        }

        ctx.encoder.copy_texture_to_texture(
            wgpu::ImageCopyTexture {
                texture: self.tex_2.as_ref().unwrap(),
                mip_level: 0,
                origin: Default::default(),
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::ImageCopyTexture {
                texture: &target.texture,
                mip_level: 0,
                origin: Default::default(),
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::Extent3d {
                width: self.size.x,
                height: self.size.y,
                depth_or_array_layers: 1,
            },
        );
    }
}
