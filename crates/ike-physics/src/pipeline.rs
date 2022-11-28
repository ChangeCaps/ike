use crossbeam::queue::SegQueue;
use ike_core::time::Time;
use ike_ecs::prelude::*;
use ike_math::Vec3;

use crate::{
    events::{Collision, CollisionData},
    rapier::{
        dynamics::{
            CCDSolver, ImpulseJointSet, IntegrationParameters, IslandManager, MultibodyJointSet,
            RigidBodySet,
        },
        geometry::{self, BroadPhase, ColliderSet, NarrowPhase},
        math::Real,
        pipeline::{EventHandler, PhysicsPipeline, QueryPipeline},
    },
    resources::Gravity,
    systems::{from_user_data, from_vec3},
};

#[derive(SystemParam)]
pub struct PhysicsWorld<'w> {
    pub query_pipeline: ResMut<'w, QueryPipeline>,
    pub colliders: ResMut<'w, ColliderSet>,
}

pub(crate) fn physics_step_system(
    event_manager: Local<EventManager>,
    mut events: EventWriter<Collision>,
    gravity: Option<Res<Gravity>>,
    time: Res<Time>,
    mut pipeline: ResMut<PhysicsPipeline>,
    mut query_pipeline: ResMut<QueryPipeline>,
    mut integration_parameters: ResMut<IntegrationParameters>,
    mut island_manager: ResMut<IslandManager>,
    mut broad_phase: ResMut<BroadPhase>,
    mut narrow_phase: ResMut<NarrowPhase>,
    mut bodies: ResMut<RigidBodySet>,
    mut colliders: ResMut<ColliderSet>,
    mut impulse_joints: ResMut<ImpulseJointSet>,
    mut multibody_joints: ResMut<MultibodyJointSet>,
    mut ccd_solver: ResMut<CCDSolver>,
) {
    let gravity = from_vec3(gravity.map_or(Vec3::ZERO, |g| g.0));

    integration_parameters.dt = time.delta();

    pipeline.step(
        &gravity,
        &integration_parameters,
        &mut island_manager,
        &mut broad_phase,
        &mut narrow_phase,
        &mut bodies,
        &mut colliders,
        &mut impulse_joints,
        &mut multibody_joints,
        &mut ccd_solver,
        &(),
        &*event_manager,
    );

    query_pipeline.update(&island_manager, &bodies, &colliders);

    event_manager.process_events(&mut events, &narrow_phase, &bodies, &colliders);
}

#[derive(Default)]
pub(crate) struct EventManager {
    queue: SegQueue<geometry::CollisionEvent>,
}

impl EventHandler for EventManager {
    #[inline]
    fn handle_collision_event(
        &self,
        _bodies: &RigidBodySet,
        _colliders: &ColliderSet,
        event: geometry::CollisionEvent,
        _contact_pair: Option<&geometry::ContactPair>,
    ) {
        self.queue.push(event);
    }

    #[inline]
    fn handle_contact_force_event(
        &self,
        _dt: Real,
        _bodies: &RigidBodySet,
        _colliders: &ColliderSet,
        _contact_pair: &geometry::ContactPair,
        _total_force_magnitude: Real,
    ) {
    }
}

impl EventManager {
    pub fn process_events(
        &self,
        event_writer: &mut EventWriter<Collision>,
        narrow_phase: &NarrowPhase,
        bodies: &RigidBodySet,
        colliders: &ColliderSet,
    ) {
        while let Some(event) = self.queue.pop() {
            let handle_a;
            let handle_b;
            let started;

            match event {
                geometry::CollisionEvent::Started(a, b, _) => {
                    handle_a = a;
                    handle_b = b;
                    started = true;
                }
                geometry::CollisionEvent::Stopped(a, b, _) => {
                    handle_a = a;
                    handle_b = b;
                    started = false;
                }
            }

            Self::data(
                event_writer,
                narrow_phase,
                bodies,
                colliders,
                started,
                handle_a,
                handle_b,
            );
        }
    }

    #[inline]
    pub fn data(
        event_writer: &mut EventWriter<Collision>,
        narrow_phase: &NarrowPhase,
        bodies: &RigidBodySet,
        colliders: &ColliderSet,
        started: bool,
        handle_a: geometry::ColliderHandle,
        handle_b: geometry::ColliderHandle,
    ) {
        let colliders = (colliders.get(handle_a), colliders.get(handle_b));

        if let (Some(collider_a), Some(collider_b)) = colliders {
            let body_a = collider_a.parent().and_then(|handle| bodies.get(handle));
            let body_b = collider_b.parent().and_then(|handle| bodies.get(handle));

            if let (Some(body_a), Some(body_b)) = (body_a, body_b) {
                if let Some(pair) = narrow_phase.contact_pair(handle_a, handle_b) {
                    for manifold in pair.manifolds.iter() {
                        let impulse: f32 = manifold.points.iter().map(|p| p.data.impulse).sum();
                        let normal = Vec3::new(
                            manifold.data.normal.x,
                            manifold.data.normal.y,
                            manifold.data.normal.z,
                        );

                        let event = Collision {
                            started,
                            a: CollisionData {
                                rigid_body: from_user_data(body_a.user_data),
                                collider: from_user_data(collider_a.user_data),
                                impulse: -impulse,
                                normal: -normal,
                            },
                            b: CollisionData {
                                rigid_body: from_user_data(body_b.user_data),
                                collider: from_user_data(collider_b.user_data),
                                impulse,
                                normal,
                            },
                        };

                        event_writer.send(event);
                    }
                }
            }
        }
    }
}
