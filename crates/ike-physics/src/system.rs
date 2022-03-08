use ike_core::Time;
use ike_ecs::{Commands, Entity, Query, Res, ResMut, Without};
use ike_math::{Quat, Vec3};
use ike_transform::GlobalTransform;
use rapier3d::{
    math::{Isometry, Translation},
    na::{ArrayStorage, Quaternion, Unit, UnitQuaternion, Vector3},
    prelude::{ColliderSet, JointSet, PhysicsPipeline, RigidBodyBuilder, RigidBodySet},
};

use crate::{Gravity, PhysicsResource, RigidBodies, RigidBody, RigidBodyHandle};

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

pub fn physics_update(
    mut physics_pipeline: ResMut<PhysicsPipeline>,
    mut physics_resource: ResMut<PhysicsResource>,
    mut rigid_body_set: ResMut<RigidBodySet>,
    mut collider_set: ResMut<ColliderSet>,
    mut joint_set: ResMut<JointSet>,
    gravity: Res<Gravity>,
    time: Res<Time>,
) {
    let physics_resource = &mut *physics_resource;

    physics_resource.integration_parameters.dt = time.delta_seconds();

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
        &(),
    );
}
