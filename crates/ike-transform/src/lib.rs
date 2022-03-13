mod component;
mod system;

pub use component::*;

use ike_app::{App, CoreStage, Plugin, StartupStage};
use ike_ecs::{ParallelSystemCoercion, SystemLabel, UpdateParentSystem};
use system::{add_global_transform_system, transform_propagate_system};

#[derive(SystemLabel, Clone, Debug, Hash)]
pub enum TransformSystem {
    AddComponents,
    Propagate,
}

#[derive(Default)]
pub struct TransformPlugin;

impl Plugin for TransformPlugin {
    fn build(self, app: &mut App) {
        app.add_startup_system_to_stage(
            system::transform_propagate_system,
            StartupStage::PostStartup,
        );

        app.add_system_to_stage(
            add_global_transform_system
                .label(TransformSystem::AddComponents)
                .after(UpdateParentSystem),
            CoreStage::PostUpdate,
        );

        app.add_system_to_stage(
            transform_propagate_system
                .label(TransformSystem::Propagate)
                .after(TransformSystem::AddComponents),
            CoreStage::PostUpdate,
        );
    }
}
