mod component;
mod system;

pub use component::*;

use ike_app::{stage, startup_stage, App, Plugin};
use ike_ecs::SystemFn;

#[derive(Default)]
pub struct TransformPlugin;

impl Plugin for TransformPlugin {
    fn build(self, app: &mut App) {
        app.add_startup_system_to_stage(
            system::transform_propagate_system.system(),
            startup_stage::POST_STARTUP,
        );

        app.add_system_to_stage(
            system::transform_propagate_system.system(),
            stage::POST_UPDATE,
        );
    }
}
