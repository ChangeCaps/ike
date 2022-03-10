use crossbeam::queue::SegQueue;
use ike_math::Vec3;
use rapier3d::prelude::{
    BroadPhase, CCDSolver, ColliderSet, ContactEvent, ContactPair, IntegrationParameters,
    IntersectionEvent, IslandManager, JointSet, NarrowPhase, PhysicsPipeline, RigidBodySet,
};

use crate::to_vec3;

pub struct PhysicsWorld {
    pub physics_pipeline: PhysicsPipeline,
    pub integration_parameters: IntegrationParameters,
    pub island_manager: IslandManager,
    pub broad_phase: BroadPhase,
    pub narrow_phase: NarrowPhase,
    pub ccd_solver: CCDSolver,
}

impl PhysicsWorld {
    pub fn new() -> Self {
        Self {
            physics_pipeline: PhysicsPipeline::new(),
            integration_parameters: IntegrationParameters::default(),
            island_manager: IslandManager::new(),
            broad_phase: BroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            ccd_solver: CCDSolver::new(),
        }
    }

    pub fn step(
        &mut self,
        gravity: Vec3,
        delta_time: f32,
        bodies: &mut RigidBodySet,
        colliders: &mut ColliderSet,
        joints: &mut JointSet,
    ) -> Vec<ContactEvent> {
        let event_handler = EventHandler::default();

        self.integration_parameters.dt = delta_time;
        self.physics_pipeline.step(
            &to_vec3(gravity),
            &self.integration_parameters,
            &mut self.island_manager,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            bodies,
            colliders,
            joints,
            &mut self.ccd_solver,
            &(),
            &event_handler,
        );

        let mut events = Vec::new();

        while let Some(event) = event_handler.contacts.pop() {
            events.push(event);
        }

        events
    }
}

#[derive(Default)]
struct EventHandler {
    intersections: SegQueue<IntersectionEvent>,
    contacts: SegQueue<ContactEvent>,
}

impl rapier3d::prelude::EventHandler for EventHandler {
    fn handle_intersection_event(&self, event: IntersectionEvent) {
        self.intersections.push(event);
    }

    fn handle_contact_event(&self, event: ContactEvent, _: &ContactPair) {
        self.contacts.push(event);
    }
}
