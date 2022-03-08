use ike_ecs::Component;
use ike_math::Vec3;

#[derive(Component, Clone, Debug)]
pub struct RigidBodyHandle(pub(crate) rapier3d::prelude::RigidBodyHandle);

#[derive(Component, Clone, Debug, Default)]
pub struct RigidBody {
    pub linear_velocity: Vec3,
    pub angular_velocity: Vec3,
    pub linear_dampening: f32,
    pub angular_dampening: f32,
    pub continuous: bool,
    pub kinematic: bool,
}

#[derive(Component, Clone, Debug, Default)]
pub struct BoxCollider {
    pub offset: Vec3,
    pub size: Vec3,
}

impl BoxCollider {}
