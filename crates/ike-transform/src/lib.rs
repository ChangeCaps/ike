mod component;
mod system;

pub use component::*;

use ike_app::{App, CoreStage, Plugin, StartupStage};

#[derive(Default)]
pub struct TransformPlugin;

impl Plugin for TransformPlugin {
    fn build(self, app: &mut App) {
        app.add_startup_system_to_stage(
            system::transform_propagate_system,
            StartupStage::PostStartup,
        );

        app.add_system_to_stage(system::transform_propagate_system, CoreStage::PostUpdate);
    }
}
