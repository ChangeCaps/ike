use std::sync::Arc;

use glam::UVec2;
use once_cell::sync::OnceCell;

pub struct RenderCtx {
    pub device: ike_wgpu::Device,
    pub queue: ike_wgpu::Queue,
}

static RENDER_CTX: OnceCell<Arc<RenderCtx>> = OnceCell::new();

#[inline]
pub fn render_device<'a>() -> &'a ike_wgpu::Device {
    &RENDER_CTX.get().expect("RENDER_CTX not set").device
}

#[inline]
pub fn render_queue<'a>() -> &'a ike_wgpu::Queue {
    &RENDER_CTX.get().expect("RENDER_CTX not set").queue
}

#[inline]
pub fn is_render_ctx_set() -> bool {
    RENDER_CTX.get().is_some()
}

#[inline]
pub fn get_render_ctx<'a>() -> &'a Arc<RenderCtx> {
    &RENDER_CTX.get().expect("RENDER_CTX not set")
}

#[inline]
pub fn set_render_ctx(render_ctx: Arc<RenderCtx>) {
    RENDER_CTX
        .set(render_ctx)
        .ok()
        .expect("RENDER_CTX already set");
}

pub enum RenderSurfaceTexture<'a> {
    Surface(ike_wgpu::SurfaceTexture),
    Texture(&'a ike_wgpu::Texture),
}

impl<'a> RenderSurfaceTexture<'a> {
    #[inline]
    pub fn texture(&self) -> &ike_wgpu::Texture {
        match self {
            Self::Surface(surface) => surface.texture(),
            Self::Texture(texture) => *texture,
        }
    }

    #[inline]
    pub fn present(self) {
        match self {
            Self::Surface(surface) => surface.present(),
            _ => {}
        }
    }
}

enum RenderSurfaceInner {
    Surface(ike_wgpu::Surface),
    Texture(ike_wgpu::Texture),
}

pub struct RenderSurface {
    inner: RenderSurfaceInner,
    config: ike_wgpu::SurfaceConfiguration,
    updated: bool,
}

impl RenderSurface {
    #[inline]
    pub fn new(surface: ike_wgpu::Surface, config: ike_wgpu::SurfaceConfiguration) -> Self {
        Self {
            inner: RenderSurfaceInner::Surface(surface),
            config,
            updated: false,
        }
    }

    #[inline]
    pub fn new_texture(config: ike_wgpu::SurfaceConfiguration) -> Self {
        let texture = render_device().create_texture(&ike_wgpu::TextureDescriptor {
            label: None,
            size: ike_wgpu::Extent3d {
                width: config.width,
                height: config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: ike_wgpu::TextureDimension::D2,
            format: config.format,
            usage: config.usage,
        });

        Self {
            inner: RenderSurfaceInner::Texture(texture),
            config,
            updated: false,
        }
    }

    #[inline]
    pub fn from_texture(
        texture: ike_wgpu::Texture,
        config: ike_wgpu::SurfaceConfiguration,
    ) -> Self {
        Self {
            inner: RenderSurfaceInner::Texture(texture),
            config,
            updated: false,
        }
    }

    #[inline]
    pub fn config(&self) -> &ike_wgpu::SurfaceConfiguration {
        &self.config
    }

    #[inline]
    pub fn size(&self) -> UVec2 {
        UVec2::new(self.config.width, self.config.height)
    }

    #[inline]
    pub fn configure(&mut self) -> &mut ike_wgpu::SurfaceConfiguration {
        self.updated = true;

        &mut self.config
    }

    #[inline]
    pub fn apply_config(&mut self) {
        if self.updated {
            self.updated = false;

            match self.inner {
                RenderSurfaceInner::Surface(ref mut surface) => {
                    surface.configure(render_device(), &self.config);
                }
                RenderSurfaceInner::Texture(ref mut texture) => {
                    *texture = render_device().create_texture(&ike_wgpu::TextureDescriptor {
                        label: Some("render_surface_texture"),
                        size: ike_wgpu::Extent3d {
                            width: self.config.width,
                            height: self.config.height,
                            depth_or_array_layers: 1,
                        },
                        mip_level_count: 1,
                        sample_count: 1,
                        dimension: ike_wgpu::TextureDimension::D2,
                        format: self.config.format,
                        usage: self.config.usage,
                    });
                }
            }
        }
    }

    #[inline]
    pub fn texture(&mut self) -> Option<&ike_wgpu::Texture> {
        self.apply_config();

        match self.inner {
            RenderSurfaceInner::Texture(ref mut texture) => Some(texture),
            _ => None,
        }
    }

    #[inline]
    pub fn next_frame(&mut self) -> Result<RenderSurfaceTexture, ike_wgpu::SurfaceError> {
        self.apply_config();

        match self.inner {
            RenderSurfaceInner::Surface(ref surface) => Ok(RenderSurfaceTexture::Surface(
                surface.get_current_texture()?,
            )),
            RenderSurfaceInner::Texture(ref texture) => Ok(RenderSurfaceTexture::Texture(texture)),
        }
    }
}
