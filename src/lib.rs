pub use ike_assets as assets;
pub use ike_core as core;
pub use ike_input as input;
pub use ike_pbr as pbr;
pub use ike_physics as physics;
pub use ike_render as render;
pub use ike_transform as transform;
pub use ike_wgpu as wgpu;
pub use ike_winit as winit;

pub mod prelude {
    pub use glam::*;
    pub use ike_assets::{AssetAppBuilderExt, Assets, Handle, HandleUntyped};
    pub use ike_core::{
        App, AppBuilder, Commands, Component, ExclusiveSystem, FnSystem, HasId, Id, Node,
        QueryMut as Query, ReadGuard, Res, ResMut, Resources, Schedule, System, Time, Without,
        World, WriteGuard, Changed
    };
    pub use ike_debug_line::{DebugLine, DebugLinePlugin};
    pub use ike_input::{Input, Mouse};
    pub use ike_pbr::{DirectionalLight, PbrMaterial, PbrPlugin, PointLight};
    pub use ike_physics::{BoxCollider, PhysicsPlugin, RigidBody};
    pub use ike_render::{
        render_device, render_queue, Buffer, Camera, Color, Color16, Color8, CubeTexture, EdgeSlot,
        EdgeSlotInfo, Environment, HdrTexture, MainCamera, Mesh, NodeEdge, OrthographicProjection,
        PerspectiveProjection, RenderCtx, RenderGraph, RenderNode, RenderPlugin, Shader, Texture,
    };
    pub use ike_transform::{
        GlobalTransform, Parent, Transform, TransformNodeExt, TransformPlugin,
    };
    pub use ike_wgpu as wgpu;
    pub use ike_winit::{Key, MouseButton, Window, WinitRunner};
}
