pub use ike_app as app;
pub use ike_assets as assets;
pub use ike_core as core;
pub use ike_debug as debug;
pub use ike_ecs as ecs;
pub use ike_id as id;
pub use ike_input as input;
pub use ike_light as light;
pub use ike_math as math;
pub use ike_node as node;
pub use ike_pbr as pbr;
#[cfg(feature = "physics")]
pub use ike_physics as physics;
#[cfg(feature = "render")]
pub use ike_render as render;
pub use ike_transform as transform;
pub use ike_winit as winit;

pub mod prelude {
    pub use crate::DefaultPlugins;
    pub use ike_app::{App, AppRunner, Plugin};
    pub use ike_assets::{AddAsset, Assets, Handle};
    pub use ike_core::Time;
    pub use ike_debug::DebugLine;
    pub use ike_ecs::{
        Added, Changed, Children, Commands, Component, Entities, Entity, EventReader, EventWriter,
        Events, ExclusiveSystemCoercion, FromWorld, Mut, Or, ParallelSystemCoercion, Parent, Query,
        Res, ResMut, Resources, Schedule, SparseStorage, StageLabel, SystemLabel, With, Without,
        World,
    };
    pub use ike_input::{Input, Key, KeyboardInput, MouseButton, MouseButtonInput};
    pub use ike_light::DirectionalLight;
    pub use ike_math::*;
    pub use ike_node::{node, AddNode, Node, NodeComponent};
    pub use ike_pbr::PbrMaterial;
    pub use ike_physics::{BoxCollider, Collision, DebugCollider, RigidBody};
    pub use ike_render::{
        Camera, Color, Mesh, RawCamera, RenderContext, RenderDevice, RenderGraph,
        RenderGraphContext, RenderGraphResult, RenderNode, RenderQueue, SlotInfo,
    };
    pub use ike_transform::{GlobalTransform, Transform, TransformPlugin};
    pub use ike_winit::Window;
}

use app::{App, Plugin};
use assets::AssetsPlugin;
use debug::DebugPlugin;
use ike_core::CorePlugin;
use input::InputPlugin;
use light::LightPlugin;
use node::NodePlugin;
use pbr::PbrPlugin;
#[cfg(feature = "physics")]
use physics::PhysicsPlugin;
#[cfg(feature = "render")]
use render::RenderPlugin;
use transform::TransformPlugin;
use winit::WinitPlugin;

pub struct DefaultPlugins;

impl Plugin for DefaultPlugins {
    fn build(self, app: &mut App) {
        app.add_plugin(CorePlugin);
        app.add_plugin(InputPlugin);
        app.add_plugin(AssetsPlugin);
        app.add_plugin(TransformPlugin);
        app.add_plugin(WinitPlugin);
        #[cfg(feature = "render")]
        app.add_plugin(RenderPlugin);
        app.add_plugin(DebugPlugin);
        #[cfg(feature = "physics")]
        app.add_plugin(PhysicsPlugin);
        app.add_plugin(NodePlugin);
        app.add_plugin(LightPlugin);
        app.add_plugin(PbrPlugin);
    }
}
