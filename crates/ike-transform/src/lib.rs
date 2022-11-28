use ike_app::{App, CoreStage, Plugin, StartupStage};
use ike_ecs::schedule::IntoSystemDescriptor;
use shiv_transform::{transform_system, TransformSystem};

pub use shiv_transform::{GlobalTransform, Transform, TransformBundle};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct TransformPlugin;

impl Plugin for TransformPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system_to_stage(
            StartupStage::PostStartup,
            transform_system.label(TransformSystem),
        );

        app.add_system_to_stage(
            CoreStage::PostUpdate,
            transform_system.label(TransformSystem),
        );
    }
}

pub mod prelude {
    pub use crate::TransformPlugin;
    pub use shiv_transform::{GlobalTransform, Transform, TransformBundle};
}
