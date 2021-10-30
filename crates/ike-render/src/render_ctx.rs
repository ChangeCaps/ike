use once_cell::sync::OnceCell;

pub struct RenderCtx {
    pub device: ike_wgpu::Device,
    pub queue: ike_wgpu::Queue,
}

static RENDER_CTX: OnceCell<RenderCtx> = OnceCell::new();

#[inline]
pub fn render_device<'a>() -> &'a ike_wgpu::Device {
    &RENDER_CTX.get().expect("RENDER_CTX not set").device
}

#[inline]
pub fn render_queue<'a>() -> &'a ike_wgpu::Queue {
    &RENDER_CTX.get().expect("RENDER_CTX not set").queue
}

#[inline]
pub fn set_render_ctx(render_ctx: RenderCtx) {
    RENDER_CTX
        .set(render_ctx)
        .ok()
        .expect("RENDER_CTX already set");
}

pub struct RenderSurface {
    surface: ike_wgpu::Surface,
    config: ike_wgpu::SurfaceConfiguration,
    updated: bool,
}

impl RenderSurface {
    #[inline]
    pub fn new(surface: ike_wgpu::Surface, config: ike_wgpu::SurfaceConfiguration) -> Self {
        Self {
            surface,
            config,
            updated: true,
        }
    }

    #[inline]
    pub fn config(&self) -> &ike_wgpu::SurfaceConfiguration {
        &self.config
    }

    #[inline]
    pub fn configure(&mut self) -> &mut ike_wgpu::SurfaceConfiguration {
        self.updated = true;

        &mut self.config
    }

    #[inline]
    pub fn surface(&self) -> &ike_wgpu::Surface {
        if self.updated {
            self.surface.configure(render_device(), &self.config);
        }

        &self.surface
    }
}
