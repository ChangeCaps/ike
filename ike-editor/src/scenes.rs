use std::{
    collections::HashMap,
    fs::read_to_string,
    path::{Path, PathBuf},
    sync::Arc,
};

use ike::{
    core::DynamicApp,
    prelude::*,
    render::{get_render_ctx, RenderSurface},
};
use libloading::Library;

pub struct LoadedScene {
    app: DynamicApp,
}

#[derive(Default)]
pub struct Scenes {
    pub current: Option<PathBuf>,
    pub scenes: HashMap<PathBuf, LoadedScene>,
    pub library: Option<Library>,
}

impl Scenes {
    #[inline]
    pub fn load(&mut self, path: impl AsRef<Path>) {
        let library = unsafe { Library::new(path.as_ref()).unwrap() };
        self.library = Some(library);
    }

    #[inline]
    pub fn loaded(&self) -> bool {
        self.library.is_some()
    }

    #[inline]
    pub fn is_open(&self) -> bool {
        self.current.is_some()
    }

    #[inline]
    pub fn current(&self) -> &LoadedScene {
        self.scenes.get(self.current.as_ref().unwrap()).unwrap()
    }

    #[inline]
    pub fn current_mut(&mut self) -> &mut LoadedScene {
        self.scenes.get_mut(self.current.as_mut().unwrap()).unwrap()
    }

    #[inline]
    pub fn current_path(&self) -> &PathBuf {
        self.current.as_ref().unwrap()
    }

    #[inline]
    pub fn current_app(&self) -> &DynamicApp {
        &self.current().app
    }

    #[inline]
    pub fn current_app_mut(&mut self) -> &mut DynamicApp {
        &mut self.current_mut().app
    }

    #[inline]
    pub fn scene_loaded(&self, path: impl AsRef<Path>) -> bool {
        self.scenes.contains_key(path.as_ref())
    }

    #[inline]
    pub fn load_scene(&mut self, path: impl Into<PathBuf>) {
        if !self.loaded() {
            panic!("app library not loaded");
        }

        let path = path.into();

        let ike_main = unsafe {
            self.library
                .as_ref()
                .unwrap()
                .get::<unsafe fn(Arc<RenderCtx>, RenderSurface) -> DynamicApp>(b"ike_main")
                .unwrap()
        };

        let render_ctx = get_render_ctx().clone();

        let render_surface = RenderSurface::new_texture(wgpu::SurfaceConfiguration {
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            width: 1,
            height: 1,
            usage: wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING,
            present_mode: wgpu::PresentMode::Fifo,
        });

        let mut app = unsafe { ike_main(render_ctx, render_surface) };

        app.execute_startup();

        if let Some(scene_str) = read_to_string(&path).ok() {
            app.world_mut().world_ref(|world| {
                let type_registry = unsafe {
                    world
                        .world()
                        .resources()
                        .read_named::<TypeRegistry>()
                        .unwrap()
                };

                let mut deserializer = ron::Deserializer::from_str(&scene_str).unwrap();

                let scene = Scene::deserialize(&mut deserializer, &type_registry).unwrap();

                scene.spawn(world.commands(), &type_registry);
            });
        } else {
            panic!("scene not found '{}'", path.display());
        }

        self.scenes.insert(path.clone(), LoadedScene { app });
        self.current = Some(path);
    }

    #[inline]
    pub fn clear(&mut self) {
        self.current.take();
        self.scenes.clear();
        self.library.take();
    }
}
