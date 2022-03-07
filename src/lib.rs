pub use ike_app as app;
pub use ike_assets as assets;
pub use ike_ecs as ecs;
pub use ike_light as light;
pub use ike_math as math;
pub use ike_pbr as pbr;
pub use ike_render as render;
pub use ike_transform as transform;
pub use ike_winit as winit;

pub mod prelude {
    pub use crate::DefaultPlugins;
    pub use ike_app::{App, AppRunner, AppStages, Plugin};
    pub use ike_assets::{AddAsset, Assets, Handle};
    pub use ike_ecs::{
        Children, Commands, Component, Entities, Entity, EventReader, EventWriter, Events,
        FromResources, Mut, Parent, Query, Res, ResMut, Resources, SparseStorage, SystemFn, With,
        Without, World,
    };
    pub use ike_light::DirectionalLight;
    pub use ike_math::*;
    pub use ike_pbr::PbrMaterial;
    pub use ike_render::{
        Camera, Color, Mesh, RawCamera, RenderContext, RenderDevice, RenderGraph,
        RenderGraphContext, RenderGraphResult, RenderNode, RenderQueue, SlotInfo,
    };
    pub use ike_transform::{GlobalTransform, Transform, TransformPlugin};
    pub use ike_winit::Window;
}

use app::{App, Plugin};
use assets::AssetsPlugin;
use light::LightPlugin;
use pbr::PbrPlugin;
use render::RenderPlugin;
use transform::TransformPlugin;
use winit::WinitPlugin;

pub struct DefaultPlugins;

impl Plugin for DefaultPlugins {
    fn build(self, app: &mut App) {
        app.add_plugin(AssetsPlugin);
        app.add_plugin(TransformPlugin);
        app.add_plugin(WinitPlugin);
        app.add_plugin(RenderPlugin);
        app.add_plugin(LightPlugin);
        app.add_plugin(PbrPlugin);
    }
}
