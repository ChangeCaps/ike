use ike_ecs::prelude::*;
use ike_math::{Mat3, Quat, Vec3};
use ike_transform::{GlobalTransform, Transform};

use nalgebra::{Quaternion, Vector3};

use crate::components::{Collider, ColliderHandle, RigidBody, RigidBodyHandle, Velocity};
use crate::rapier::{
    dynamics::{ImpulseJointSet, IslandManager, MultibodyJointSet, RigidBodySet},
    geometry::ColliderSet,
    math::{Isometry, Rotation, Translation},
    pipeline::ActiveEvents,
};

#[inline]
fn to_rapier(t: Vec3, r: Quat) -> Isometry<f32> {
    let translation = Translation::new(t.x, t.y, t.z);
    let rotation = Rotation::new_normalize(Quaternion::new(r.w, r.x, r.y, r.z));
    Isometry::from_parts(translation, rotation)
}

#[inline]
pub(crate) const fn to_user_data(entity: Entity) -> u128 {
    entity.index() as u128 + ((entity.generation() as u128) << 32)
}

#[inline]
pub(crate) const fn from_user_data(data: u128) -> Entity {
    Entity::from_raw_parts(data as u32, (data >> 32) as u32)
}

#[inline]
pub(crate) fn from_vec3(v: Vec3) -> Vector3<f32> {
    Vector3::new(v.x, v.y, v.z)
}

#[inline]
pub(crate) fn _to_vec3(v: Vector3<f32>) -> Vec3 {
    Vec3::new(v.x, v.y, v.z)
}

#[inline]
fn to_ike(t: Isometry<f32>) -> (Vec3, Quat) {
    let translation = t.translation.vector;
    let rotation = t.rotation.coords;
    (
        Vec3::new(translation.x, translation.y, translation.z),
        Quat::from_xyzw(rotation.x, rotation.y, rotation.z, rotation.w),
    )
}

pub(crate) fn add_rigid_body_system(
    mut commands: Commands,
    mut bodies: ResMut<RigidBodySet>,
    query: Query<
        (Entity, &GlobalTransform, &RigidBody, Option<&Velocity>),
        Without<RigidBodyHandle>,
    >,
) {
    for (entity, transform, body, velocity) in query.iter() {
        let (_, rotation, translation) = transform.compute_matrix().to_scale_rotation_translation();

        let mut builder = body
            .builder()
            .user_data(to_user_data(entity))
            .position(to_rapier(translation, rotation));

        if let Some(velocity) = velocity {
            builder = builder
                .linvel(from_vec3(velocity.linear))
                .angvel(from_vec3(velocity.angular));
        }

        let handle = bodies.insert(builder.build());
        commands.entity(entity).insert(RigidBodyHandle(handle));
    }
}

pub(crate) fn add_collider_system(
    mut commands: Commands,
    mut bodies: ResMut<RigidBodySet>,
    mut colliders: ResMut<ColliderSet>,
    body_query: Query<(&RigidBody, &RigidBodyHandle)>,
    query: Query<(Entity, &Collider, Option<&Parent>), Without<ColliderHandle>>,
) {
    for (entity, collider, _parent) in query.iter() {
        if let Some((_body, handle)) = body_query.get(entity) {
            let builder = collider
                .builder()
                .user_data(to_user_data(entity))
                .active_events(ActiveEvents::all());

            let handle = colliders.insert_with_parent(builder.build(), handle.0, &mut bodies);
            commands.entity(entity).insert(ColliderHandle(handle));
        }
    }
}

pub(crate) fn update_rapier_rigid_body_system(
    mut bodies: ResMut<RigidBodySet>,
    mut query: Query<
        (
            &RigidBodyHandle,
            (&RigidBody, Changed<RigidBody>),
            (&GlobalTransform, Changed<GlobalTransform>),
            Option<(&Velocity, Changed<Velocity>)>,
        ),
        Or<(
            Changed<RigidBody>,
            Changed<GlobalTransform>,
            Changed<Velocity>,
        )>,
    >,
) {
    for (handle, rigid_body, transform, velocity) in query.iter_mut() {
        if let Some(body) = bodies.get_mut(handle.0) {
            if let (rigid_body, true) = rigid_body {
                body.set_body_type(rigid_body.body_type());
            }

            if let (transform, true) = transform {
                let (_, rotation, translation) =
                    transform.compute_matrix().to_scale_rotation_translation();

                if body.is_kinematic() {
                    body.set_next_kinematic_position(to_rapier(translation, rotation));
                } else {
                    body.set_position(to_rapier(translation, rotation), true);
                }
            }

            if let Some((velocity, true)) = velocity {
                body.set_linvel(from_vec3(velocity.linear), true);
                body.set_angvel(from_vec3(velocity.angular), true);
            }
        }
    }
}

pub(crate) fn update_ike_position_system(
    bodies: Res<RigidBodySet>,
    mut query: Query<(
        Option<&mut Transform>,
        &mut GlobalTransform,
        &RigidBodyHandle,
    )>,
) {
    for (local, mut global, rigid_body_handle) in query.iter_mut() {
        let body = match bodies.get(rigid_body_handle.0) {
            Some(body) => body,
            None => continue,
        };

        let (translation, rotation) = to_ike(*body.position());

        let (global_scale, global_rotation, global_translation) =
            global.compute_matrix().to_scale_rotation_translation();

        if translation == global_translation && rotation == global_rotation {
            continue;
        }

        if let Some(mut local) = local {
            if local.translation == global_translation {
                local.translation = translation;
            } else {
                local.translation = translation - (global_translation - local.translation);
            }

            if local.rotation == global_rotation {
                local.rotation = rotation;
            } else {
                local.rotation =
                    rotation * (global_rotation * local.rotation.conjugate()).conjugate();
            }
        }

        global.translation = translation;
        global.matrix = Mat3::from_quat(rotation) * Mat3::from_diagonal(global_scale);
    }
}

pub(crate) fn remove_body_system(
    mut commands: Commands,
    mut bodies: ResMut<RigidBodySet>,
    mut colliders: ResMut<ColliderSet>,
    mut island_manager: ResMut<IslandManager>,
    mut impulse_joints: ResMut<ImpulseJointSet>,
    mut multibody_joints: ResMut<MultibodyJointSet>,
    query: Query<(), With<RigidBody>>,
) {
    let mut removed = Vec::new();

    for (handle, body) in bodies.iter() {
        let entity = from_user_data(body.user_data);

        if !query.contains(entity) {
            removed.push(handle);
        }
    }

    for handle in removed {
        if let Some(body) = bodies.get(handle).cloned() {
            let entity = from_user_data(body.user_data);

            if let Some(mut entity) = commands.get_entity(entity) {
                entity.remove::<RigidBodyHandle>();
            }

            for &collider_handle in body.colliders() {
                let entity = from_user_data(colliders.get(collider_handle).unwrap().user_data);

                if let Some(mut entity) = commands.get_entity(entity) {
                    entity.remove::<ColliderHandle>();
                }

                colliders.remove(collider_handle, &mut island_manager, &mut bodies, false);
            }

            bodies.remove(
                handle,
                &mut island_manager,
                &mut colliders,
                &mut impulse_joints,
                &mut multibody_joints,
                true,
            );
        }
    }
}

pub(crate) fn remove_collider_system(
    mut commands: Commands,
    mut colliders: ResMut<ColliderSet>,
    mut island_manager: ResMut<IslandManager>,
    mut bodies: ResMut<RigidBodySet>,
    query: Query<(), With<Collider>>,
) {
    let mut removed = Vec::new();

    for (handle, collider) in colliders.iter() {
        let entity = from_user_data(collider.user_data);

        if !query.contains(entity) {
            removed.push(handle);
        }
    }

    for handle in removed {
        if let Some(collider) = colliders.get(handle) {
            let entity = from_user_data(collider.user_data);

            if let Some(mut entity) = commands.get_entity(entity) {
                entity.remove::<ColliderHandle>();
            }

            colliders.remove(handle, &mut island_manager, &mut bodies, true);
        }
    }
}
