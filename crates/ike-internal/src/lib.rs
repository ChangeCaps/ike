pub use ike_app as app;
pub use ike_ecs as ecs;
pub use ike_input as input;
pub use ike_lumi as lumi;
pub use ike_transform as transform;
pub use ike_wgpu as wgpu;
pub use ike_window as window;

pub mod prelude {
    pub use crate::app::{App, AppExit, AppRunner, CoreStage, Plugin, Plugins, StartupStage};
    pub use crate::ecs::prelude::*;
    pub use crate::input::*;
    pub use crate::lumi::prelude::*;
    pub use crate::transform::{GlobalTransform, Transform, TransformBundle, TransformPlugin};
    pub use crate::window::{Window, WindowClosed, WindowId, WindowPlugin, Windows};
    pub use crate::DefaultPlugins;
}

use app::{App, Plugin, Plugins};
use lumi::LumiPlugin;
use transform::TransformPlugin;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct DefaultPlugins;

impl Plugin for DefaultPlugins {
    fn build(&self, _app: &mut App) {}

    fn dependencies(&self, plugins: &mut Plugins) {
        plugins.add(TransformPlugin);
        plugins.add(LumiPlugin);
    }
}
