use crossbeam::queue::SegQueue;
use ike_core::Time;
use ike_ecs::{Changed, Commands, Entity, EventWriter, Or, Query, Res, ResMut, With, Without};
use ike_math::{Quat, Vec3};
use ike_transform::{GlobalTransform, Transform};
use rapier3d::{
    math::{Isometry, Translation},
    na::{ArrayStorage, Quaternion, Unit, UnitQuaternion, Vector3},
    prelude::{
        ActiveEvents, ColliderBuilder, ColliderSet, ContactEvent, ContactPair, IntersectionEvent,
        JointSet, PhysicsPipeline, RigidBodyBuilder, RigidBodySet, SharedShape,
    },
};

use crate::{
    BoxCollider, ColliderHandle, Colliders, Collision, Gravity, PhysicsResource, RigidBodies,
    RigidBody, RigidBodyHandle,
};

#[inline]
fn to_vec3(vec3: Vec3) -> Vector3<f32> {
    Vector3::from_data(ArrayStorage([[vec3.x, vec3.y, vec3.z]]))
}

#[inline]
fn from_vec3(vec: Vector3<f32>) -> Vec3 {
    Vec3::new(vec.x, vec.y, vec.z)
}

#[inline]
fn to_quat(quat: Quat) -> Quaternion<f32> {
    Quaternion::new(quat.w, quat.x, quat.y, quat.z)
}

#[inline]
fn from_quat(quat: UnitQuaternion<f32>) -> Quat {
    Quat::from_xyzw(quat.i, quat.j, quat.k, quat.w)
}

pub fn add_rigid_bodies(
    commands: Commands,
    mut rigid_body_set: ResMut<RigidBodySet>,
    mut rigid_bodies: ResMut<RigidBodies>,
    query: Query<(Entity, &RigidBody, &GlobalTransform), Without<RigidBodyHandle>>,
) {
    for (entity, rigid_body, transform) in query.iter() {
        let rigid_body = if rigid_body.kinematic {
            RigidBodyBuilder::new_kinematic_position_based()
                .position(Isometry::from_parts(
                    Translation::from(to_vec3(transform.translation)),
                    Unit::new_normalize(to_quat(transform.rotation)),
                ))
                .build()
        } else {
            RigidBodyBuilder::new_dynamic()
                .position(Isometry::from_parts(
                    Translation::from(to_vec3(transform.translation)),
                    Unit::new_normalize(to_quat(transform.rotation)),
                ))
                .build()
        };

        let handle = RigidBodyHandle(rigid_body_set.insert(rigid_body));
        rigid_bodies.0.insert(handle.0, entity);

        commands.insert(&entity, handle);
    }
}

pub fn add_box_colliders(
    commands: Commands,
    mut collider_set: ResMut<ColliderSet>,
    mut rigid_body_set: ResMut<RigidBodySet>,
    mut colliders: ResMut<Colliders>,
    query: Query<
        (Entity, &BoxCollider, &GlobalTransform, &RigidBodyHandle),
        Without<ColliderHandle>,
    >,
) {
    for (entity, box_collider, transform, rigid_body_handle) in query.iter() {
        let collider = ColliderBuilder::cuboid(
            box_collider.size.x / 2.0 * transform.scale.x,
            box_collider.size.y / 2.0 * transform.scale.y,
            box_collider.size.z / 2.0 * transform.scale.z,
        )
        .active_events(ActiveEvents::CONTACT_EVENTS)
        .build();

        let handle =
            collider_set.insert_with_parent(collider, rigid_body_handle.0, &mut rigid_body_set);
        colliders.0.insert(handle, entity);

        commands.insert(&entity, ColliderHandle(handle));
    }
}

pub fn set_rigid_bodies(
    mut rigid_body_set: ResMut<RigidBodySet>,
    transform_query: Query<
        (&GlobalTransform, &RigidBodyHandle),
        Or<Changed<GlobalTransform>, Changed<RigidBodyHandle>>,
    >,
    rigid_body_query: Query<
        (&RigidBody, &RigidBodyHandle),
        Or<Changed<RigidBody>, Changed<RigidBodyHandle>>,
    >,
) {
    for (transform, rigid_body_handle) in transform_query.iter() {
        let rigid_body = rigid_body_set.get_mut(rigid_body_handle.0).unwrap();

        rigid_body.set_position(
            Isometry::from_parts(
                Translation::from(to_vec3(transform.translation)),
                Unit::new_normalize(to_quat(transform.rotation)),
            ),
            true,
        );
    }

    for (rigid_body, rigid_body_handle) in rigid_body_query.iter() {
        let rb = rigid_body_set.get_mut(rigid_body_handle.0).unwrap();

        rb.set_linvel(to_vec3(rigid_body.linear_velocity), true);
        rb.set_angvel(to_vec3(rigid_body.angular_velocity), true);
        rb.set_linear_damping(rigid_body.linear_dampening);
        rb.set_angular_damping(rigid_body.angular_dampening);
        rb.enable_ccd(rigid_body.continuous);
    }
}

pub fn set_box_colliders(
    mut collider_set: ResMut<ColliderSet>,
    query: Query<
        (&GlobalTransform, &BoxCollider, &ColliderHandle),
        Or<Changed<GlobalTransform>, Changed<BoxCollider>>,
    >,
) {
    for (transform, box_collider, collider_handle) in query.iter() {
        let collider = collider_set.get_mut(collider_handle.0).unwrap();

        collider.set_shape(SharedShape::cuboid(
            box_collider.size.x / 2.0 * transform.scale.x,
            box_collider.size.y / 2.0 * transform.scale.y,
            box_collider.size.z / 2.0 * transform.scale.z,
        ));
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

pub fn physics_update(
    mut physics_pipeline: ResMut<PhysicsPipeline>,
    mut physics_resource: ResMut<PhysicsResource>,
    mut rigid_body_set: ResMut<RigidBodySet>,
    mut collider_set: ResMut<ColliderSet>,
    mut joint_set: ResMut<JointSet>,
    gravity: Res<Gravity>,
    time: Res<Time>,
    colliders: Res<Colliders>,
    mut event_writer: EventWriter<Collision>,
) {
    let physics_resource = &mut *physics_resource;

    physics_resource.integration_parameters.dt = time.delta_seconds();

    let event_handler = EventHandler::default();

    physics_pipeline.step(
        &to_vec3(gravity.0),
        &physics_resource.integration_parameters,
        &mut physics_resource.island_manager,
        &mut physics_resource.broad_phase,
        &mut physics_resource.narrow_phase,
        &mut rigid_body_set,
        &mut collider_set,
        &mut joint_set,
        &mut physics_resource.ccd_solver,
        &(),
        &event_handler,
    );

    for event in event_handler.contacts.into_iter() {
        let event = match event {
            ContactEvent::Started(a, b) => Collision::Started(colliders.0[&a], colliders.0[&b]),
            ContactEvent::Stopped(a, b) => Collision::Stopped(colliders.0[&a], colliders.0[&b]),
        };

        event_writer.send(event);
    }
}

pub fn get_rigid_bodies(
    rigid_body_set: Res<RigidBodySet>,
    mut query: Query<(
        &mut Transform,
        &mut GlobalTransform,
        &mut RigidBody,
        &RigidBodyHandle,
    )>,
) {
    for (mut transform, mut global_transform, mut rigid_body, rigid_body_handle) in query.iter_mut()
    {
        let rb = rigid_body_set.get(rigid_body_handle.0).unwrap();

        let position = rb.position();
        let translation = from_vec3(position.translation.vector);
        let rotation = from_quat(position.rotation).normalize();

        let local_rotation = global_transform.rotation * transform.rotation.conjugate().normalize();
        let inverse_local_rotation = local_rotation.conjugate();

        let local_translation = inverse_local_rotation
            * (translation
                - (global_transform.translation - local_rotation * transform.translation));

        let transform = transform.as_mut_unchanged();
        transform.translation = local_translation;
        transform.rotation = local_rotation;

        let global_transform = global_transform.as_mut_unchanged();
        global_transform.translation = translation;
        global_transform.rotation = rotation;

        let rigid_body = rigid_body.as_mut_unchanged();
        rigid_body.linear_velocity = from_vec3(*rb.linvel());
        rigid_body.angular_velocity = from_vec3(*rb.angvel());
    }
}

pub fn clean_physics(
    mut physics_resource: ResMut<PhysicsResource>,
    mut rigid_body_set: ResMut<RigidBodySet>,
    mut collider_set: ResMut<ColliderSet>,
    mut joint_set: ResMut<JointSet>,
    mut rigid_bodies: ResMut<RigidBodies>,
    rigid_body_query: Query<Entity, With<RigidBodyHandle>>,
    mut colliders: ResMut<Colliders>,
    collider_query: Query<Entity, With<ColliderHandle>>,
) {
    rigid_bodies.0.retain(|handle, entity| {
        if !rigid_body_query.contains(entity) {
            rigid_body_set.remove(
                *handle,
                &mut physics_resource.island_manager,
                &mut collider_set,
                &mut joint_set,
            );

            false
        } else {
            true
        }
    });

    colliders
        .0
        .retain(|_, entity| collider_query.contains(entity));
}
