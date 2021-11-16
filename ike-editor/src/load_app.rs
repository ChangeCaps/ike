use std::{path::Path, sync::Arc};

use ike::{core::DynamicApp, prelude::*, render::RenderSurface};
use libloading::Library;

pub struct LoadedApp {
    pub app: DynamicApp,
    pub library: Library,
}

impl LoadedApp {
    #[inline]
    pub unsafe fn load(path: impl AsRef<Path>) -> Result<Self, libloading::Error> {
        let library = unsafe { Library::new(path.as_ref())? };

        let symbol = unsafe {
            library.get::<unsafe fn(Arc<RenderCtx>, RenderSurface) -> DynamicApp>(b"ike_main")?
        };

        let render_ctx = ike::render::get_render_ctx().clone();
        let render_surface = RenderSurface::new_texture(wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            width: 1,
            height: 1,
            present_mode: wgpu::PresentMode::Immediate,
        });

        let mut app = unsafe { symbol(render_ctx, render_surface) };

        app.execute_startup();

        Ok(LoadedApp { app, library })
    }
}
