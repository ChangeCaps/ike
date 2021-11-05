use glam::Vec3;

#[derive(Clone, Debug, Default)]
pub struct RigidBody {
    pub linear_velocity: Vec3,
    pub angular_velocity: Vec3,
    pub linear_dampening: f32,
    pub angular_dampening: f32,
    pub continuous: bool,
    pub kinematic: bool,
}

impl RigidBody {
    #[inline]
    pub fn kinematic() -> Self {
        Self {
            kinematic: true,
            ..Default::default()
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct BoxCollider {
    pub size: Vec3,
}

impl BoxCollider {
    #[inline]
    pub fn new(size: Vec3) -> Self {
        Self { size }
    }
}
