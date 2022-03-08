use std::collections::HashMap;

use ike_ecs::Entity;
use ike_math::Vec3;
use rapier3d::prelude::{
    BroadPhase, CCDSolver, ColliderHandle, IntegrationParameters, IslandManager, NarrowPhase,
    RigidBodyHandle,
};

pub struct PhysicsResource {
    pub integration_parameters: IntegrationParameters,
    pub island_manager: IslandManager,
    pub broad_phase: BroadPhase,
    pub narrow_phase: NarrowPhase,
    pub ccd_solver: CCDSolver,
}

impl Default for PhysicsResource {
    fn default() -> Self {
        Self {
            integration_parameters: Default::default(),
            island_manager: IslandManager::new(),
            broad_phase: BroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            ccd_solver: CCDSolver::new(),
        }
    }
}

#[derive(Default)]
pub struct RigidBodies(pub HashMap<RigidBodyHandle, Entity>);

#[derive(Default)]
pub struct Colliders(pub HashMap<ColliderHandle, Entity>);

pub struct Gravity(pub Vec3);

impl Default for Gravity {
    fn default() -> Self {
        Self(Vec3::Y * -9.81)
    }
}
