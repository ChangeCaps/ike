pub use ike_app as app;
pub use ike_core as core;
pub use ike_ecs as ecs;
pub use ike_input as input;
pub use ike_lumi as lumi;
pub use ike_math as math;
pub use ike_physics as physics;
pub use ike_transform as transform;
pub use ike_wgpu as wgpu;
pub use ike_window as window;

pub mod prelude {
    pub use crate::app::prelude::*;
    pub use crate::core::prelude::*;
    pub use crate::ecs::prelude::*;
    pub use crate::input::*;
    pub use crate::lumi::{prelude::*, LumiPlugin, RenderPlugin, RenderPluginAppExt, RenderStage};
    pub use crate::math::*;
    pub use crate::physics::prelude::*;
    pub use crate::transform::prelude::*;
    pub use crate::window::prelude::*;
    pub use crate::DefaultPlugins;
}

use app::{App, Plugin, Plugins};
use ike_core::CorePlugin;
use lumi::LumiPlugin;
use physics::PhysicsPlugin;
use transform::TransformPlugin;
use wgpu::WgpuPlugin;
use window::WindowPlugin;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct DefaultPlugins;

impl Plugin for DefaultPlugins {
    fn build(&self, _app: &mut App) {}

    fn dependencies(&self, plugins: &mut Plugins) {
        plugins.add(CorePlugin);
        plugins.add(TransformPlugin);
        plugins.add(PhysicsPlugin);
        plugins.add(WindowPlugin);
        plugins.add(WgpuPlugin);
        plugins.add(LumiPlugin);
    }
}
