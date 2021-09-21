use once_cell::sync::OnceCell;

use crate::{
    id::{HasId, Id},
    prelude::HdrTexture,
    renderer::RenderCtx,
};

pub struct CubeTexture {
    id: Id<Self>,
    size: u32,
    texture: OnceCell<ike_wgpu::Texture>,
}

impl HasId<CubeTexture> for CubeTexture {
    #[inline]
    fn id(&self) -> Id<CubeTexture> {
        self.id
    }
}

impl Default for CubeTexture {
    #[inline]
    fn default() -> Self {
        Self {
            id: Id::new(),
            size: 1,
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
    pub fn load_hdr_texture(&mut self, ctx: &RenderCtx, hdr_texture: &HdrTexture) {
        let size = hdr_texture.height() / 2;

        let eq_texture = hdr_texture.texture(ctx).create_view(&Default::default());

        let cube_texture = ctx.device.create_texture(&ike_wgpu::TextureDescriptor {
            label: None,
            size: ike_wgpu::Extent3d {
                width: size,
                height: size,
                depth_or_array_layers: 6,
            },
            format: ike_wgpu::TextureFormat::Rgba32Float,
            mip_level_count: 1,
            sample_count: 1,
            dimension: ike_wgpu::TextureDimension::D2,
            usage: ike_wgpu::TextureUsages::STORAGE_BINDING
                | ike_wgpu::TextureUsages::TEXTURE_BINDING,
        });

        let eq_layout = ctx
            .device
            .create_bind_group_layout(&ike_wgpu::BindGroupLayoutDescriptor {
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

        let eq_group = ctx
            .device
            .create_bind_group(&ike_wgpu::BindGroupDescriptor {
                label: None,
                layout: &eq_layout,
                entries: &[ike_wgpu::BindGroupEntry {
                    binding: 0,
                    resource: ike_wgpu::BindingResource::TextureView(&eq_texture),
                }],
            });

        let cube_layout =
            ctx.device
                .create_bind_group_layout(&ike_wgpu::BindGroupLayoutDescriptor {
                    label: None,
                    entries: &[ike_wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        ty: ike_wgpu::BindingType::StorageTexture {
                            access: ike_wgpu::StorageTextureAccess::WriteOnly,
                            format: ike_wgpu::TextureFormat::Rgba32Float,
                            view_dimension: ike_wgpu::TextureViewDimension::D2Array,
                        },
                        visibility: ike_wgpu::ShaderStages::COMPUTE,
                        count: None,
                    }],
                });

        let view = cube_texture.create_view(&Default::default());

        let cube_group = ctx
            .device
            .create_bind_group(&ike_wgpu::BindGroupDescriptor {
                label: None,
                layout: &cube_layout,
                entries: &[ike_wgpu::BindGroupEntry {
                    binding: 0,
                    resource: ike_wgpu::BindingResource::TextureView(&view),
                }],
            });

        let layout = ctx
            .device
            .create_pipeline_layout(&ike_wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[&eq_layout, &cube_layout],
                push_constant_ranges: &[],
            });

        let module = ctx
            .device
            .create_shader_module(&ike_wgpu::include_wgsl!("eq_to_cube.comp.wgsl"));

        let pipeline = ctx
            .device
            .create_compute_pipeline(&ike_wgpu::ComputePipelineDescriptor {
                label: None,
                layout: Some(&layout),
                module: &module,
                entry_point: "main",
            });

        let mut encoder = ctx.device.create_command_encoder(&Default::default());

        let mut compute_pass = encoder.begin_compute_pass(&Default::default());

        compute_pass.set_pipeline(&pipeline);

        compute_pass.set_bind_group(0, &eq_group, &[]);
        compute_pass.set_bind_group(1, &cube_group, &[]);

        compute_pass.dispatch(size / 8, size / 8, 6);

        drop(compute_pass);

        ctx.queue.submit_once(encoder.finish());

        self.size = size;
        self.texture = OnceCell::from(cube_texture);
    }

    #[inline]
    pub fn inner(&self, ctx: &RenderCtx) -> &ike_wgpu::Texture {
        self.texture.get_or_init(|| {
            ctx.device.create_texture(&ike_wgpu::TextureDescriptor {
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
    pub fn view(&self, ctx: &RenderCtx) -> ike_wgpu::TextureView {
        self.inner(ctx)
            .create_view(&ike_wgpu::TextureViewDescriptor {
                dimension: Some(ike_wgpu::TextureViewDimension::Cube),
                ..Default::default()
            })
    }
}
