pub use ike_app as app;
pub use ike_assets as assets;
pub use ike_core as core;
pub use ike_debug as debug;
pub use ike_ecs as ecs;
#[cfg(feature = "gltf")]
pub use ike_gltf as gltf;
pub use ike_input as input;
pub use ike_light as light;
use ike_log::LogPlugin;
pub use ike_math as makth;
pub use ike_node as node;
#[cfg(feature = "pbr")]
pub use ike_pbr as pbr;
#[cfg(feature = "physics")]
pub use ike_physics as physics;
#[cfg(feature = "render")]
pub use ike_render as render;
pub use ike_task as task;
pub use ike_transform as transform;
pub use ike_util as util;
pub use ike_winit as winit;

pub mod prelude {
    pub use crate::DefaultPlugins;
    pub use ike_app::{App, AppRunner, Plugin};
    pub use ike_assets::{AddAsset, AssetServer, Assets, Handle};
    pub use ike_core::Time;
    pub use ike_debug::DebugLine;
    pub use ike_ecs::{
        Added, Changed, Children, Commands, Component, Entities, Entity, EventReader, EventWriter,
        Events, ExclusiveSystemCoercion, FromWorld, Mut, Or, ParallelSystemCoercion, Parent, Query,
        Res, ResMut, Resources, Schedule, SparseStorage, StageLabel, SystemLabel, With, Without,
        World,
    };
    #[cfg(feature = "gltf")]
    pub use ike_gltf::GltfMesh;
    pub use ike_input::{Input, Key, KeyboardInput, MouseButton, MouseButtonInput};
    pub use ike_light::DirectionalLight;
    pub use ike_math::*;
    pub use ike_node::{node, AddNode, Node, NodeComponent};
    #[cfg(feature = "pbr")]
    pub use ike_pbr::PbrMaterial;
    #[cfg(feature = "physics")]
    pub use ike_physics::{BoxCollider, Collision, DebugCollider, Gravity, RigidBody};
    #[cfg(feature = "render")]
    pub use ike_render::{
        Camera, Color, Mesh, RawCamera, RenderContext, RenderDevice, RenderGraph,
        RenderGraphContext, RenderGraphResult, RenderNode, RenderQueue, SlotInfo,
    };
    pub use ike_task::{Task, TaskPool};
    pub use ike_transform::{GlobalTransform, Transform, TransformPlugin};
    pub use ike_winit::Window;
}

use app::{App, Plugin};
use assets::AssetsPlugin;
use debug::DebugPlugin;
use ike_core::CorePlugin;
#[cfg(feature = "gltf")]
use ike_gltf::GltfPlugin;
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
        app.add_plugin(LogPlugin);
        app.add_plugin(InputPlugin);
        app.add_plugin(AssetsPlugin);
        app.add_plugin(TransformPlugin);
        app.add_plugin(WinitPlugin);
        #[cfg(feature = "render")]
        app.add_plugin(RenderPlugin);
        app.add_plugin(DebugPlugin);
        #[cfg(feature = "physics")]
        app.add_plugin(PhysicsPlugin);
        #[cfg(feature = "gltf")]
        app.add_plugin(GltfPlugin);
        app.add_plugin(NodePlugin);
        app.add_plugin(LightPlugin);
        #[cfg(feature = "pbr")]
        app.add_plugin(PbrPlugin);
    }
}
