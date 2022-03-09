mod component;
mod debug;
mod event;
mod resource;
mod system;

pub use component::*;
pub use debug::*;
pub use event::*;
pub use rapier3d::prelude::{ColliderSet, JointSet, PhysicsPipeline, RigidBodySet};
pub use resource::*;
pub use system::*;

use ike_app::{App, CoreStage, Plugin};
use ike_ecs::{ParallelSystemCoercion, StageLabel, SystemLabel};

#[derive(StageLabel, Clone, Copy, Debug, Hash)]
pub enum PhysicsStage {
    PrePhysics,
    Physics,
    PostPhysics,
}

#[derive(SystemLabel, Clone, Copy, Debug, Hash)]
pub enum PhysicsSystem {
    AddComponents,
    SetComponents,
    GetComponents,
    Clean,
    Debug,
}

#[derive(Default)]
pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(self, app: &mut App) {
        app.init_resource::<PhysicsPipeline>();
        app.init_resource::<PhysicsResource>();
        app.insert_resource(RigidBodySet::new());
        app.insert_resource(ColliderSet::new());
        app.insert_resource(JointSet::new());
        app.init_resource::<RigidBodies>();
        app.init_resource::<Colliders>();
        app.init_resource::<Gravity>();

        app.add_event::<Collision>();

        app.add_stage_after(PhysicsStage::PrePhysics, CoreStage::PostUpdate);
        app.add_stage_after(PhysicsStage::Physics, PhysicsStage::PrePhysics);
        app.add_stage_after(PhysicsStage::PostPhysics, PhysicsStage::Physics);

        app.add_system_to_stage(
            add_rigid_bodies.label(PhysicsSystem::AddComponents),
            PhysicsStage::PrePhysics,
        );

        app.add_system_to_stage(
            add_box_colliders.label(PhysicsSystem::AddComponents),
            PhysicsStage::PrePhysics,
        );

        app.add_system_to_stage(
            set_rigid_bodies
                .label(PhysicsSystem::GetComponents)
                .after(PhysicsSystem::AddComponents),
            PhysicsStage::PrePhysics,
        );
        app.add_system_to_stage(
            set_box_colliders
                .label(PhysicsSystem::GetComponents)
                .after(PhysicsSystem::AddComponents),
            PhysicsStage::PrePhysics,
        );

        app.add_system_to_stage(physics_update, PhysicsStage::Physics);

        app.add_system_to_stage(
            get_rigid_bodies.label(PhysicsSystem::GetComponents),
            PhysicsStage::PostPhysics,
        );

        app.add_system_to_stage(
            debug_box_collider_system
                .label(PhysicsSystem::Debug)
                .after(PhysicsSystem::GetComponents),
            PhysicsStage::PostPhysics,
        );
    }
}
