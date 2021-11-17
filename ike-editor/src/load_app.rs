use std::{fs::read_to_string, path::Path, sync::Arc};

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

        if let Some(scene_str) = read_to_string("scene.scn").ok() {
            let type_registry = unsafe {
                app.world()
                    .resources()
                    .read_named::<TypeRegistry>()
                    .unwrap()
            };

            let mut deserialiser = ron::Deserializer::from_str(&scene_str).unwrap();

            let scene = Scene::deserialize(&mut deserialiser, &type_registry).unwrap();

            drop(type_registry);

            app.world_mut().world_ref(|world| {
                let type_registry = unsafe {
                    world
                        .world()
                        .resources()
                        .read_named::<TypeRegistry>()
                        .unwrap()
                };

                scene.spawn(world.commands(), &type_registry);
            });
        }

        Ok(LoadedApp { app, library })
    }
}
