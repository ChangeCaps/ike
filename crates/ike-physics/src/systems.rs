use glam::{Quat, Vec3};
use ike_core::*;
use ike_transform::{GlobalTransform, Transform};
use rapier3d::{math::{Isometry, Translation}, na::{ArrayStorage, Quaternion, Unit, UnitQuaternion, Vector3}, prelude::{
        ColliderBuilder, ColliderHandle, ColliderSet, JointSet, RigidBodyBuilder, RigidBodyHandle,
        RigidBodySet,
    }};

use crate::{BoxCollider, Colliders, Gravity, PhysicsResource, RigidBodies, RigidBody};

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
    for (entity, rigid_body, transform) in query {
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

        let handle = rigid_body_set.insert(rigid_body);
        rigid_bodies.0.insert(handle, entity);

        commands.insert_component(&entity, handle);
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
    for (entity, collider, global_transform, rigid_body) in query {
        let collider = ColliderBuilder::cuboid(
            collider.size.x / 2.0 * global_transform.scale.x,
            collider.size.y / 2.0 * global_transform.scale.y,
            collider.size.z / 2.0 * global_transform.scale.z,
        )
        .build();

        let handle = collider_set.insert_with_parent(collider, *rigid_body, &mut rigid_body_set);
        colliders.0.insert(handle, entity);

        commands.insert_component(&entity, handle);
    }
}

pub fn set_rigid_bodies(
    commands: Commands,
    mut physics_resource: ResMut<PhysicsResource>,
    mut rigid_body_set: ResMut<RigidBodySet>,
    mut collider_set: ResMut<ColliderSet>,
    mut joint_set: ResMut<JointSet>,
    transform_query: Query<
        (&GlobalTransform, &RigidBodyHandle),
        Or<Changed<GlobalTransform>, Changed<RigidBodyHandle>>,
    >,
    rigid_body_query: Query<
        (Entity, &RigidBody, &RigidBodyHandle),
        Or<Changed<RigidBody>, Changed<RigidBodyHandle>>,
    >,
) {
    for (global_transform, rigid_body_handle) in transform_query {
        let rigid_body = rigid_body_set.get_mut(*rigid_body_handle).unwrap();

        rigid_body.set_position(
            Isometry::from_parts(
                Translation::from(to_vec3(global_transform.translation)),
                Unit::new_normalize(to_quat(global_transform.rotation)),
            ),
            true,
        );
    } 

    for (entity, rigid_body, rigid_body_handle) in rigid_body_query {
        let rb = rigid_body_set.get_mut(*rigid_body_handle).unwrap();

        rb.set_linvel(to_vec3(rigid_body.linear_velocity), true);
        rb.set_angvel(to_vec3(rigid_body.angular_velocity), true);
        rb.set_linear_damping(rigid_body.linear_dampening);
        rb.set_angular_damping(rigid_body.angular_dampening);
        rb.restrict_rotations(
            rigid_body.angular_lock.x,
            rigid_body.angular_lock.y,
            rigid_body.angular_lock.z,
            true,
        );
        rb.enable_ccd(rigid_body.continuous);

        if rb.is_kinematic() != rigid_body.kinematic {
            commands.remove_component::<RigidBodyHandle>(&entity);
            commands.remove_component::<ColliderHandle>(&entity);

            rigid_body_set.remove(
                *rigid_body_handle,
                &mut physics_resource.island_manager,
                &mut collider_set,
                &mut joint_set,
            );
        }
    }
}

pub fn physics_update(
    mut physics_resource: ResMut<PhysicsResource>,
    mut rigid_body_set: ResMut<RigidBodySet>,
    mut collider_set: ResMut<ColliderSet>,
    mut joint_set: ResMut<JointSet>,
    gravity: Res<Gravity>,
    time: Res<Time>,
) {
    let physics_resource = &mut *physics_resource;

    physics_resource.integration_parameters.dt = time.delta_time();

    physics_resource.pipeline.step(
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
        &(),
    );
}

pub fn get_rigid_bodies(
    rigid_body_set: Res<RigidBodySet>,
    query: Query<(
        &mut Transform,
        &mut GlobalTransform,
        &mut RigidBody,
        &RigidBodyHandle,
    )>,
) {
    for (mut transform, mut global_transform, mut rigid_body, rigid_body_handle) in query {
        let rb = rigid_body_set.get(*rigid_body_handle).unwrap();

        let position = rb.position();
        let translation = from_vec3(position.translation.vector);
        let rotation = from_quat(position.rotation).normalize();

        let rot = global_transform.rotation * transform.rotation.conjugate().normalize();
        let inv_rot = rot.conjugate().normalize();

        {
            let translation = inv_rot
                * (translation - (global_transform.translation - rot * transform.translation));

            transform.unmarked().translation = translation;

            let rotation = rotation * inv_rot;

            transform.unmarked().rotation = rotation;
        }

        global_transform.unmarked().translation = translation;
        global_transform.unmarked().rotation = rotation;

        rigid_body.unmarked().linear_velocity = from_vec3(*rb.linvel());
        rigid_body.unmarked().angular_velocity = from_vec3(*rb.angvel());
    }
}

pub fn clean_physics(
    mut physics_resource: ResMut<PhysicsResource>,
    mut rigid_body_set: ResMut<RigidBodySet>,
    mut collider_set: ResMut<ColliderSet>,
    mut joint_set: ResMut<JointSet>,
    mut rigid_bodies: ResMut<RigidBodies>,
    mut rigid_body_query: Query<Entity, With<RigidBodyHandle>>,
    mut colliders: ResMut<Colliders>,
    mut collider_query: Query<Entity, With<ColliderHandle>>,
) {
    rigid_bodies.0.retain(|handle, entity| {
        if rigid_body_query.get(*entity).is_none() {
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

    colliders.0.retain(|_handle, entity| {
        collider_query.get(*entity).is_some()
    });
}
