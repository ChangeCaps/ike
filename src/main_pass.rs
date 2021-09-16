use crate::{
    prelude::{Color, View},
    renderer::{PassData, RenderCtx, RenderPass, SampleCount, TargetFormat, TargetSize},
};

pub struct MainPass {
    pub clear_color: Color,
    pub sample_count: u32,
    width: u32,
    height: u32,
    depth_texture: Option<ike_wgpu::TextureView>,
    ms_texture: Option<ike_wgpu::TextureView>,
}

impl Default for MainPass {
    #[inline]
    fn default() -> Self {
        Self {
            clear_color: Color::BLACK,
            sample_count: 1,
            width: 0,
            height: 0,
            depth_texture: None,
            ms_texture: None,
        }
    }
}

impl<S> RenderPass<S> for MainPass {
    fn run<'a>(
        &'a mut self,
        encoder: &'a mut ike_wgpu::CommandEncoder,
        ctx: &RenderCtx,
        view: &'a View,
        data: &mut PassData,
        _state: &mut S,
    ) -> ike_wgpu::RenderPass<'a> {
        data.insert(SampleCount(self.sample_count));
        data.insert(TargetFormat(view.format));
        data.insert(TargetSize {
            width: view.width,
            height: view.height,
        });
        data.insert(view.camera.clone());

        let depth = if let Some(ref mut depth) = self.depth_texture {
            if self.width != view.width || self.height != view.height {
                let texture = ctx.device.create_texture(&ike_wgpu::TextureDescriptor {
                    label: None,
                    size: ike_wgpu::Extent3d {
                        width: view.width,
                        height: view.height,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: self.sample_count,
                    dimension: ike_wgpu::TextureDimension::D2,
                    format: ike_wgpu::TextureFormat::Depth24Plus,
                    usage: ike_wgpu::TextureUsages::RENDER_ATTACHMENT,
                });

                if self.sample_count > 1 {
                    let ms_texture = ctx.device.create_texture(&ike_wgpu::TextureDescriptor {
                        label: None,
                        size: ike_wgpu::Extent3d {
                            width: view.width,
                            height: view.height,
                            depth_or_array_layers: 1,
                        },
                        mip_level_count: 1,
                        sample_count: self.sample_count,
                        dimension: ike_wgpu::TextureDimension::D2,
                        format: view.format,
                        usage: ike_wgpu::TextureUsages::RENDER_ATTACHMENT,
                    });

                    self.ms_texture = Some(ms_texture.create_view(&Default::default()));
                }

                self.width = view.width;
                self.height = view.height;

                let view = texture.create_view(&Default::default());
                *depth = view;
            }

            depth
        } else {
            let depth_texture = ctx.device.create_texture(&ike_wgpu::TextureDescriptor {
                label: None,
                size: ike_wgpu::Extent3d {
                    width: view.width,
                    height: view.height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: self.sample_count,
                dimension: ike_wgpu::TextureDimension::D2,
                format: ike_wgpu::TextureFormat::Depth24Plus,
                usage: ike_wgpu::TextureUsages::RENDER_ATTACHMENT,
            });

            if self.sample_count > 1 {
                let ms_texture = ctx.device.create_texture(&ike_wgpu::TextureDescriptor {
                    label: None,
                    size: ike_wgpu::Extent3d {
                        width: view.width,
                        height: view.height,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: self.sample_count,
                    dimension: ike_wgpu::TextureDimension::D2,
                    format: view.format,
                    usage: ike_wgpu::TextureUsages::RENDER_ATTACHMENT,
                });

                self.ms_texture = Some(ms_texture.create_view(&Default::default()));
            }

            self.width = view.width;
            self.height = view.height;

            self.depth_texture = Some(depth_texture.create_view(&Default::default()));

            self.depth_texture.as_ref().unwrap()
        };

        let color_attachment = if self.sample_count > 1 {
            ike_wgpu::RenderPassColorAttachment {
                view: self.ms_texture.as_ref().unwrap(),
                resolve_target: Some(&view.target),
                ops: ike_wgpu::Operations {
                    load: ike_wgpu::LoadOp::Clear(self.clear_color.into()),
                    store: true,
                },
            }
        } else {
            ike_wgpu::RenderPassColorAttachment {
                view: &view.target,
                resolve_target: None,
                ops: ike_wgpu::Operations {
                    load: ike_wgpu::LoadOp::Clear(self.clear_color.into()),
                    store: true,
                },
            }
        };

        let desc = ike_wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[color_attachment],
            depth_stencil_attachment: Some(ike_wgpu::RenderPassDepthStencilAttachment {
                view: depth,
                depth_ops: Some(ike_wgpu::Operations {
                    load: ike_wgpu::LoadOp::Clear(1.0),
                    store: true,
                }),
                stencil_ops: None,
            }),
        };

        encoder.begin_render_pass(&desc)
    }
}
