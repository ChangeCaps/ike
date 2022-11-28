pub mod components;
pub mod events;
pub mod pipeline;
pub mod resources;
pub mod systems;

#[cfg(feature = "3d")]
use rapier3d as rapier;

use ike_app::{
    app::{App, CoreStage},
    plugin::{Plugin, Plugins},
};
use ike_ecs::schedule::{Schedule, StageLabel, SystemStage};
use ike_transform::TransformPlugin;

#[derive(Clone, Copy, Debug, StageLabel)]
pub enum PhysicsStage {
    Physics,
    Step,
}

#[derive(Clone, Copy, Debug, StageLabel)]
enum SubStage {
    Remove,
    AddBodies,
    AddColliders,
    UpdateRapier,
    Step,
    UpdateIke,
}

fn remove() -> SystemStage {
    SystemStage::parallel()
        .with_system(systems::remove_body_system)
        .with_system(systems::remove_collider_system)
}

fn add_bodies() -> SystemStage {
    SystemStage::parallel().with_system(systems::add_rigid_body_system)
}

fn add_colliders() -> SystemStage {
    SystemStage::parallel().with_system(systems::add_collider_system)
}

fn update_raiper() -> SystemStage {
    SystemStage::parallel().with_system(systems::update_rapier_rigid_body_system)
}

fn step_stage() -> SystemStage {
    SystemStage::parallel().with_system(pipeline::physics_step_system)
}

fn update_ike() -> SystemStage {
    SystemStage::parallel()
        .with_system(systems::update_ike_position_system)
        .with_system(systems::update_ike_velocity_system)
}

fn step() -> Schedule {
    let mut schedule = Schedule::default();

    schedule.add_stage(SubStage::Remove, remove());
    schedule.add_stage(SubStage::AddBodies, add_bodies());
    schedule.add_stage(SubStage::AddColliders, add_colliders());
    schedule.add_stage(SubStage::UpdateRapier, update_raiper());
    schedule.add_stage(SubStage::Step, step_stage());
    schedule.add_stage(SubStage::UpdateIke, update_ike());

    schedule
}

#[derive(Clone, Copy, Debug, Default)]
pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<rapier::pipeline::PhysicsPipeline>();
        app.init_resource::<rapier::pipeline::QueryPipeline>();
        app.init_resource::<rapier::dynamics::IntegrationParameters>();
        app.init_resource::<rapier::dynamics::IslandManager>();
        app.init_resource::<rapier::geometry::BroadPhase>();
        app.init_resource::<rapier::geometry::NarrowPhase>();
        app.init_resource::<rapier::dynamics::RigidBodySet>();
        app.init_resource::<rapier::geometry::ColliderSet>();
        app.init_resource::<rapier::dynamics::ImpulseJointSet>();
        app.init_resource::<rapier::dynamics::MultibodyJointSet>();
        app.init_resource::<rapier::dynamics::CCDSolver>();

        app.add_event::<events::Collision>();

        app.add_stage_after(
            CoreStage::Update,
            PhysicsStage::Physics,
            SystemStage::parallel(),
        );
        app.add_stage_after(PhysicsStage::Physics, PhysicsStage::Step, step());
    }

    fn dependencies(&self, plugins: &mut Plugins) {
        plugins.add(TransformPlugin);
    }
}

pub mod prelude {
    pub use crate::components::{Collider, RigidBody, Velocity};
    pub use crate::events::{Collision, CollisionData};
    pub use crate::pipeline::PhysicsWorld;
    pub use crate::resources::Gravity;
    pub use crate::{PhysicsPlugin, PhysicsStage};
}
