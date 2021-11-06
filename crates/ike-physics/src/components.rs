use glam::Vec3;
use ike_render::Color;

#[derive(Clone, Copy, Debug, Default)]
pub struct LockAxis {
    pub x: bool,
    pub y: bool,
    pub z: bool,
}

#[derive(Clone, Debug, Default)]
pub struct RigidBody {
    pub linear_velocity: Vec3,
    pub angular_velocity: Vec3,
    pub linear_dampening: f32,
    pub angular_dampening: f32,
    pub angular_lock: LockAxis,
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
    pub debug: Option<Color>,
}

impl BoxCollider {
    #[inline]
    pub fn new(size: Vec3) -> Self {
        Self { size, debug: None }
    }
}
