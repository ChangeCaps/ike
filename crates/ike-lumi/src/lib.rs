use ike_app::{
    app::{App, CoreStage},
    plugin::{Plugin, Plugins},
};
use ike_ecs::{
    event::EventReader,
    query::Query,
    schedule::{IntoSystemDescriptor, StageLabel, SystemLabel, SystemStage},
    system::{Res, ResMut},
    world::{Entity, World},
};
use ike_transform::TransformPlugin;
use ike_wgpu::{WgpuPlugin, WindowSurfaces};
use ike_window::{WindowRedrawRequested, Windows};

use lumi::{
    core::{Device, Queue, RenderTarget},
    renderer::{Camera, CameraTarget, Renderer, RendererPlugin},
    DefaultPlugin,
};

pub use lumi::*;

fn lumi_extract_system(world: &mut World) {
    let device = world.remove_resource::<Device>().unwrap();
    let queue = world.remove_resource::<Queue>().unwrap();
    let mut renderer = world.remove_resource::<Renderer>().unwrap();

    renderer.extract(&device, &queue, world);

    world.insert_resource(device);
    world.insert_resource(queue);
    world.insert_resource(renderer);
}

fn lumi_render_system(
    mut redraw_events: EventReader<WindowRedrawRequested>,
    mut renderer: ResMut<Renderer>,
    device: Res<Device>,
    queue: Res<Queue>,
    windows: Res<Windows>,
    window_surfaces: Res<WindowSurfaces>,
    camera_query: Query<(Entity, &Camera)>,
) {
    let requested = redraw_events
        .iter()
        .map(|e| e.window_id)
        .collect::<Vec<_>>();

    let mut priorities = Vec::new();

    for (entity, camera) in camera_query.iter() {
        priorities.push((entity, camera.priority));
    }

    priorities.sort_by_key(|&(_, priority)| priority);

    for (entity, _) in priorities {
        let (_, camera) = camera_query.get(entity).unwrap();

        match camera.target {
            CameraTarget::Texture(ref texture) => {
                let target = RenderTarget {
                    view: texture,
                    width: texture.size().width,
                    height: texture.size().height,
                };

                renderer.render(&device, &queue, entity, target);
            }
            CameraTarget::Main => {
                if !requested.contains(&windows.primary_id()) {
                    continue;
                }

                let window = windows.primary();
                let surface = window_surfaces.get(&windows.primary_id()).unwrap();

                let texture = surface.surface().get_current_texture().unwrap();
                let view = texture.texture.create_view(&Default::default());

                let (width, height) = window.get_size();
                let target = RenderTarget {
                    view: &view,
                    width,
                    height,
                };

                renderer.render(&device, &queue, entity, target);

                texture.present();
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct RenderPlugin<T: RendererPlugin>(pub T);

impl<T: RendererPlugin + Send + Sync + 'static> Plugin for RenderPlugin<T> {
    fn build(&self, app: &mut App) {
        let mut renderer = app.world.resource_mut::<Renderer>();
        renderer.add_plugin(&self.0);
    }

    fn dependencies(&self, plugins: &mut Plugins) {
        plugins.add(LumiPlugin);
    }
}

pub trait RenderPluginAppExt {
    fn add_render_plugin<T: RendererPlugin + Send + Sync + 'static>(
        &mut self,
        plugin: T,
    ) -> &mut Self;
}

impl RenderPluginAppExt for App {
    fn add_render_plugin<T: RendererPlugin + Send + Sync + 'static>(
        &mut self,
        plugin: T,
    ) -> &mut Self {
        self.add_plugin(RenderPlugin(plugin))
    }
}

#[derive(Clone, Copy, Debug, StageLabel)]
pub enum RenderStage {
    PreRender,
    Render,
    PostRender,
}

#[derive(Clone, Copy, Debug, SystemLabel)]
pub enum LumiSystem {
    Extract,
    Render,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct LumiPlugin;

impl Plugin for LumiPlugin {
    fn build(&self, app: &mut App) {
        app.add_stage_after(
            CoreStage::PostUpdate,
            RenderStage::PreRender,
            SystemStage::parallel(),
        );
        app.add_stage_after(
            RenderStage::PreRender,
            RenderStage::Render,
            SystemStage::parallel(),
        );
        app.add_stage_after(
            RenderStage::Render,
            RenderStage::PostRender,
            SystemStage::parallel(),
        );

        let mut renderer = Renderer::new();
        renderer.add_plugin(DefaultPlugin);

        app.add_resource(renderer);

        app.add_system_to_stage(
            RenderStage::PreRender,
            lumi_extract_system.label(LumiSystem::Extract),
        );
        app.add_system_to_stage(
            RenderStage::Render,
            lumi_render_system.label(LumiSystem::Render),
        );
    }

    fn dependencies(&self, plugins: &mut Plugins) {
        plugins.add(WgpuPlugin);
        plugins.add(TransformPlugin);
    }
}
