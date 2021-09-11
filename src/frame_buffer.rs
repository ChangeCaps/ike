use crate::renderer::RenderCtx;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FrameBufferDescriptor {
    pub width: u32,
    pub height: u32,
    pub format: ike_wgpu::TextureFormat,
    pub usage: ike_wgpu::TextureUsages,
}

impl Default for FrameBufferDescriptor {
    #[inline]
    fn default() -> Self {
        Self {
            width: 1,
            height: 1,
            format: ike_wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: ike_wgpu::TextureUsages::RENDER_ATTACHMENT,
        }
    }
}

#[derive(Default)]
pub struct FrameBuffer {
    pub descriptor: FrameBufferDescriptor,
    current_descriptor: FrameBufferDescriptor,
    version: u64,
    color: Option<ike_wgpu::Texture>,
    depth: Option<ike_wgpu::Texture>,
}

impl FrameBuffer {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn version(&self) -> u64 {
        self.version
    }

    #[inline]
    pub fn color(&mut self, ctx: &'_ RenderCtx) -> &ike_wgpu::Texture {
        if self.current_descriptor != self.descriptor {
            self.color = None;
            self.depth = None;

            self.current_descriptor = self.descriptor.clone();

            self.version += 1;
        }

        if self.color.is_none() {
            let texture = ctx.device.create_texture(&ike_wgpu::TextureDescriptor {
                label: None,
                size: ike_wgpu::Extent3d {
                    width: self.descriptor.width,
                    height: self.descriptor.height,
                    depth_or_array_layers: 1,
                },
                dimension: ike_wgpu::TextureDimension::D2,
                mip_level_count: 1,
                sample_count: 1,
                format: self.descriptor.format,
                usage: self.descriptor.usage,
            });

            self.color = Some(texture);
        }

        self.color.as_ref().unwrap()
    }

    #[inline]
    pub fn depth(&mut self, ctx: &'_ RenderCtx) -> &ike_wgpu::Texture {
        if self.current_descriptor != self.descriptor {
            self.color = None;
            self.depth = None;

            self.current_descriptor = self.descriptor.clone();

            self.version += 1;
        }

        if self.depth.is_none() {
            let texture = ctx.device.create_texture(&ike_wgpu::TextureDescriptor {
                label: None,
                size: ike_wgpu::Extent3d {
                    width: self.descriptor.width,
                    height: self.descriptor.height,
                    depth_or_array_layers: 1,
                },
                dimension: ike_wgpu::TextureDimension::D2,
                mip_level_count: 1,
                sample_count: 1,
                format: ike_wgpu::TextureFormat::Depth24Plus,
                usage: self.descriptor.usage,
            });

            self.depth = Some(texture);
        }

        self.depth.as_ref().unwrap()
    }

    #[inline]
    pub fn color_depth(&mut self, ctx: &'_ RenderCtx) -> (&ike_wgpu::Texture, &ike_wgpu::Texture) {
        self.color(ctx);
        self.depth(ctx);

        (self.color.as_ref().unwrap(), self.depth.as_ref().unwrap())
    }
}
