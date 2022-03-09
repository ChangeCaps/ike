use ike_ecs::Component;
use ike_math::Vec3;

#[derive(Component, Clone, Debug)]
pub struct RigidBodyHandle(pub(crate) rapier3d::prelude::RigidBodyHandle);

#[derive(Component, Clone, Debug)]
pub struct ColliderHandle(pub(crate) rapier3d::prelude::ColliderHandle);

#[derive(Component, Clone, Debug, Default)]
pub struct RigidBody {
    pub linear_velocity: Vec3,
    pub angular_velocity: Vec3,
    pub linear_dampening: f32,
    pub angular_dampening: f32,
    pub continuous: bool,
    pub kinematic: bool,
}

impl RigidBody {
    pub fn dynamic() -> Self {
        Self::default()
    }

    pub fn kinematic() -> Self {
        Self {
            kinematic: true,
            ..Default::default()
        }
    }
}

#[derive(Component, Clone, Copy, Debug)]
pub struct DebugCollider;

#[derive(Component, Clone, Debug)]
pub struct BoxCollider {
    pub size: Vec3,
}

impl Default for BoxCollider {
    fn default() -> Self {
        Self { size: Vec3::ONE }
    }
}

impl BoxCollider {
    pub const fn new(size: Vec3) -> Self {
        Self { size }
    }
}
