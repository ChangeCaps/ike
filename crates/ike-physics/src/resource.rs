use std::collections::HashMap;

use ike_ecs::Entity;
use ike_math::Vec3;
use rapier3d::prelude::{ColliderHandle, RigidBodyHandle};

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
