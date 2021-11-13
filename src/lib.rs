pub use ike_assets as assets;
pub use ike_core as core;
pub use ike_debug_line as debug_line;
pub use ike_input as input;
#[cfg(feature = "pbr")]
pub use ike_pbr as pbr;
#[cfg(feature = "physics")]
pub use ike_physics as physics;
pub use ike_reflect as reflect;
pub use ike_render as render;
pub use ike_scene as scene;
pub use ike_transform as transform;
pub use ike_wgpu as wgpu;
pub use ike_winit as winit;

pub mod prelude {
    pub use crate::DefaultPlugins;
    pub use glam::*;
    pub use ike_assets::{AssetAppBuilderExt, Assets, Handle, HandleUntyped};
    pub use ike_core::{
        App, AppBuilder, Changed, Commands, Component, Entity, ExclusiveSystem, FnSystem, HasId,
        Id, Node, Or, Query, ReadGuard, Res, ResMut, Resources, Schedule, SpawnNode, System, Time,
        With, Without, World, WorldRef, WriteGuard,
    };
    pub use ike_debug_line::{DebugLine, DebugLinePlugin};
    pub use ike_input::{Input, Mouse};

    #[cfg(feature = "pbr")]
    pub use ike_pbr::{DirectionalLight, PbrMaterial, PbrPlugin, PointLight};

    #[cfg(feature = "physics")]
    pub use ike_physics::{BoxCollider, PhysicsPlugin, RigidBody};
    pub use ike_reflect::{FromReflect, Reflect, ReflectAppBuilderExt, TypeRegistry};
    pub use ike_render::{
        render_device, render_queue, Buffer, Camera, Color, Color16, Color8, CubeTexture, EdgeSlot,
        EdgeSlotInfo, Environment, HdrTexture, MainCamera, Mesh, NodeEdge, OrthographicProjection,
        PerspectiveProjection, RenderCtx, RenderGraph, RenderNode, RenderPlugin, Shader, Texture,
    };
    pub use ike_scene::{Scene, SceneNode};
    pub use ike_transform::{
        GlobalTransform, Parent, Transform, TransformNodeExt, TransformPlugin,
    };
    pub use ike_wgpu as wgpu;
    pub use ike_winit::{Key, MouseButton, Window, WinitRunner};
}

use ike_core::*;

pub struct DefaultPlugins;

impl Plugin for DefaultPlugins {
    #[inline]
    fn build(self, app: &mut AppBuilder) {
        app.set_runner(winit::WinitRunner);
        app.add_plugin(render::RenderPlugin);
        app.add_plugin(debug_line::DebugLinePlugin);

        #[cfg(feature = "pbr")]
        app.add_plugin(pbr::PbrPlugin);
        app.add_plugin(transform::TransformPlugin);

        #[cfg(feature = "physics")]
        app.add_plugin(physics::PhysicsPlugin);
        app.add_plugin(scene::ScenePlugin);
    }
}
