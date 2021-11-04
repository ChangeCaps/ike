use bytemuck::bytes_of;
use once_cell::sync::OnceCell;

use crate::{render_device, render_queue, HdrTexture};

pub struct Environment {
    pub env_texture: CubeTexture,
    pub irradiance_texture: CubeTexture,
}

impl Default for Environment {
    #[inline]
    fn default() -> Self {
        Self {
            env_texture: CubeTexture::new(1024),
            irradiance_texture: CubeTexture::new(128),
        }
    }
}

impl Environment {
    pub fn load(&mut self, hdr_texture: &HdrTexture) {
        self.env_texture.load_hdr_texture(hdr_texture);
        self.irradiance_texture.load_irradiance(hdr_texture);
    }
}

pub struct CubeTexture {
    size: u32,
    texture: OnceCell<ike_wgpu::Texture>,
}

impl Default for CubeTexture {
    #[inline]
    fn default() -> Self {
        Self {
            #[cfg(debug_assertions)]
            size: 512,
            #[cfg(not(debug_assertions))]
            size: 1024,
            texture: OnceCell::new(),
        }
    }
}

impl CubeTexture {
    #[inline]
    pub fn new(size: u32) -> Self {
        Self {
            size,
            ..Default::default()
        }
    }

    #[inline]
    pub fn load_hdr_texture(&mut self, hdr_texture: &HdrTexture) {
        let eq_texture = hdr_texture.texture().create_view(&Default::default());
        let stages = 1;

        let device = render_device();

        let cube_texture = device.create_texture(&ike_wgpu::TextureDescriptor {
            label: None,
            size: ike_wgpu::Extent3d {
                width: self.size,
                height: self.size,
                depth_or_array_layers: 6,
            },
            format: ike_wgpu::TextureFormat::Rgba32Float,
            mip_level_count: 1,
            sample_count: 1,
            dimension: ike_wgpu::TextureDimension::D2,
            usage: ike_wgpu::TextureUsages::STORAGE_BINDING
                | ike_wgpu::TextureUsages::TEXTURE_BINDING,
        });

        let eq_layout = device.create_bind_group_layout(&ike_wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[ike_wgpu::BindGroupLayoutEntry {
                binding: 0,
                ty: ike_wgpu::BindingType::Texture {
                    sample_type: ike_wgpu::TextureSampleType::Float { filterable: false },
                    view_dimension: ike_wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                visibility: ike_wgpu::ShaderStages::COMPUTE,
                count: None,
            }],
        });

        let eq_group = device.create_bind_group(&ike_wgpu::BindGroupDescriptor {
            label: None,
            layout: &eq_layout,
            entries: &[ike_wgpu::BindGroupEntry {
                binding: 0,
                resource: ike_wgpu::BindingResource::TextureView(&eq_texture),
            }],
        });

        let cube_layout = device.create_bind_group_layout(&ike_wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                ike_wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    ty: ike_wgpu::BindingType::StorageTexture {
                        access: ike_wgpu::StorageTextureAccess::WriteOnly,
                        format: ike_wgpu::TextureFormat::Rgba32Float,
                        view_dimension: ike_wgpu::TextureViewDimension::D2Array,
                    },
                    visibility: ike_wgpu::ShaderStages::COMPUTE,
                    count: None,
                },
                ike_wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    ty: ike_wgpu::BindingType::Buffer {
                        ty: ike_wgpu::BufferBindingType::Uniform,
                        min_binding_size: None,
                        has_dynamic_offset: false,
                    },
                    visibility: ike_wgpu::ShaderStages::COMPUTE,
                    count: None,
                },
            ],
        });

        let cube_groups = (0..stages)
            .into_iter()
            .map(|i| {
                let view = cube_texture.create_view(&ike_wgpu::TextureViewDescriptor {
                    base_array_layer: 6 / stages * i,
                    ..Default::default()
                });

                let buffer = device.create_buffer_init(&ike_wgpu::BufferInitDescriptor {
                    label: None,
                    contents: bytes_of(&(6 / stages * i)),
                    usage: ike_wgpu::BufferUsages::COPY_DST | ike_wgpu::BufferUsages::UNIFORM,
                });

                device.create_bind_group(&ike_wgpu::BindGroupDescriptor {
                    label: None,
                    layout: &cube_layout,
                    entries: &[
                        ike_wgpu::BindGroupEntry {
                            binding: 0,
                            resource: ike_wgpu::BindingResource::TextureView(&view),
                        },
                        ike_wgpu::BindGroupEntry {
                            binding: 1,
                            resource: buffer.as_entire_binding(),
                        },
                    ],
                })
            })
            .collect::<Vec<_>>();

        let layout = device.create_pipeline_layout(&ike_wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&eq_layout, &cube_layout],
            push_constant_ranges: &[],
        });

        let module = device.create_shader_module(&ike_wgpu::include_wgsl!("eq_to_cube.comp.wgsl"));

        let pipeline = device.create_compute_pipeline(&ike_wgpu::ComputePipelineDescriptor {
            label: None,
            layout: Some(&layout),
            module: &module,
            entry_point: "main",
        });

        for cube_group in cube_groups {
            let mut encoder = device.create_command_encoder(&Default::default());

            let mut compute_pass = encoder.begin_compute_pass(&Default::default());

            compute_pass.set_pipeline(&pipeline);

            compute_pass.set_bind_group(0, &eq_group, &[]);

            compute_pass.set_bind_group(1, &cube_group, &[]);

            compute_pass.dispatch(self.size / 32, self.size / 32, 6 / stages);

            drop(compute_pass);

            render_queue().submit_once(encoder.finish());
        }

        self.texture = OnceCell::from(cube_texture);
    }

    #[inline]
    pub fn load_irradiance(&mut self, hdr_texture: &HdrTexture) {
        let eq_texture = hdr_texture.texture().create_view(&Default::default());
        let stages = 6;

        let device = render_device();

        let cube_texture = device.create_texture(&ike_wgpu::TextureDescriptor {
            label: None,
            size: ike_wgpu::Extent3d {
                width: self.size,
                height: self.size,
                depth_or_array_layers: 6,
            },
            format: ike_wgpu::TextureFormat::Rgba32Float,
            mip_level_count: 1,
            sample_count: 1,
            dimension: ike_wgpu::TextureDimension::D2,
            usage: ike_wgpu::TextureUsages::STORAGE_BINDING
                | ike_wgpu::TextureUsages::TEXTURE_BINDING,
        });

        let eq_layout = device.create_bind_group_layout(&ike_wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[ike_wgpu::BindGroupLayoutEntry {
                binding: 0,
                ty: ike_wgpu::BindingType::Texture {
                    sample_type: ike_wgpu::TextureSampleType::Float { filterable: false },
                    view_dimension: ike_wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                visibility: ike_wgpu::ShaderStages::COMPUTE,
                count: None,
            }],
        });

        let eq_group = device.create_bind_group(&ike_wgpu::BindGroupDescriptor {
            label: None,
            layout: &eq_layout,
            entries: &[ike_wgpu::BindGroupEntry {
                binding: 0,
                resource: ike_wgpu::BindingResource::TextureView(&eq_texture),
            }],
        });

        let cube_layout = device.create_bind_group_layout(&ike_wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                ike_wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    ty: ike_wgpu::BindingType::StorageTexture {
                        access: ike_wgpu::StorageTextureAccess::WriteOnly,
                        format: ike_wgpu::TextureFormat::Rgba32Float,
                        view_dimension: ike_wgpu::TextureViewDimension::D2Array,
                    },
                    visibility: ike_wgpu::ShaderStages::COMPUTE,
                    count: None,
                },
                ike_wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    ty: ike_wgpu::BindingType::Buffer {
                        ty: ike_wgpu::BufferBindingType::Uniform,
                        min_binding_size: None,
                        has_dynamic_offset: false,
                    },
                    visibility: ike_wgpu::ShaderStages::COMPUTE,
                    count: None,
                },
            ],
        });

        let cube_groups = (0..stages)
            .into_iter()
            .map(|i| {
                let view = cube_texture.create_view(&ike_wgpu::TextureViewDescriptor {
                    base_array_layer: 6 / stages * i,
                    ..Default::default()
                });

                let buffer = device.create_buffer_init(&ike_wgpu::BufferInitDescriptor {
                    label: None,
                    contents: bytes_of(&(6 / stages * i)),
                    usage: ike_wgpu::BufferUsages::COPY_DST | ike_wgpu::BufferUsages::UNIFORM,
                });

                device.create_bind_group(&ike_wgpu::BindGroupDescriptor {
                    label: None,
                    layout: &cube_layout,
                    entries: &[
                        ike_wgpu::BindGroupEntry {
                            binding: 0,
                            resource: ike_wgpu::BindingResource::TextureView(&view),
                        },
                        ike_wgpu::BindGroupEntry {
                            binding: 1,
                            resource: buffer.as_entire_binding(),
                        },
                    ],
                })
            })
            .collect::<Vec<_>>();

        let layout = device.create_pipeline_layout(&ike_wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&eq_layout, &cube_layout],
            push_constant_ranges: &[],
        });

        let module =
            device.create_shader_module(&ike_wgpu::include_wgsl!("eq_to_irradiance.comp.wgsl"));

        let pipeline = device.create_compute_pipeline(&ike_wgpu::ComputePipelineDescriptor {
            label: None,
            layout: Some(&layout),
            module: &module,
            entry_point: "main",
        });

        for cube_group in cube_groups {
            let mut encoder = device.create_command_encoder(&Default::default());

            let mut compute_pass = encoder.begin_compute_pass(&Default::default());

            compute_pass.set_pipeline(&pipeline);

            compute_pass.set_bind_group(0, &eq_group, &[]);

            compute_pass.set_bind_group(1, &cube_group, &[]);

            compute_pass.dispatch(self.size / 32, self.size / 32, 6 / stages);

            drop(compute_pass);

            render_queue().submit_once(encoder.finish());
        }

        self.texture = OnceCell::from(cube_texture);
    }

    #[inline]
    pub fn inner(&self) -> &ike_wgpu::Texture {
        self.texture.get_or_init(|| {
            render_device().create_texture(&ike_wgpu::TextureDescriptor {
                label: None,
                size: ike_wgpu::Extent3d {
                    width: self.size,
                    height: self.size,
                    depth_or_array_layers: 6,
                },
                format: ike_wgpu::TextureFormat::Rgba32Float,
                mip_level_count: 1,
                sample_count: 1,
                dimension: ike_wgpu::TextureDimension::D2,
                usage: ike_wgpu::TextureUsages::STORAGE_BINDING
                    | ike_wgpu::TextureUsages::TEXTURE_BINDING,
            })
        })
    }

    #[inline]
    pub fn view(&self) -> ike_wgpu::TextureView {
        self.inner().create_view(&ike_wgpu::TextureViewDescriptor {
            dimension: Some(ike_wgpu::TextureViewDimension::Cube),
            ..Default::default()
        })
    }
}
