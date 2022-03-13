use ike_ecs::Component;
use ike_math::Vec3;
use rapier3d::{na::Point3, prelude::SharedShape};

#[derive(Component, Clone, Debug)]
pub struct RigidBodyHandle(pub(crate) rapier3d::prelude::RigidBodyHandle);

#[derive(Component, Clone, Debug)]
pub struct ColliderHandle(pub(crate) rapier3d::prelude::ColliderHandle);

#[derive(Clone, Debug, Default)]
pub struct LockAxis {
    pub x: bool,
    pub y: bool,
    pub z: bool,
}

#[derive(Component, Clone, Debug, Default)]
pub struct RigidBody {
    pub linear_velocity: Vec3,
    pub angular_velocity: Vec3,
    pub linear_dampening: f32,
    pub angular_dampening: f32,
    pub lock_translation: bool,
    pub lock_rotation: LockAxis,
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

    pub fn with_rotation_lock(mut self, locked: bool) -> Self {
        self.lock_rotation.x = locked;
        self.lock_rotation.y = locked;
        self.lock_rotation.z = locked;
        self
    }

    pub fn with_lock_x(mut self, locked: bool) -> Self {
        self.lock_rotation.x = locked;
        self
    }

    pub fn with_lock_y(mut self, locked: bool) -> Self {
        self.lock_rotation.y = locked;
        self
    }

    pub fn with_lock_z(mut self, locked: bool) -> Self {
        self.lock_rotation.z = locked;
        self
    }

    pub fn with_continuous(mut self, continuous: bool) -> Self {
        self.continuous = continuous;
        self
    }
}

#[derive(Component, Clone, Copy, Debug)]
pub struct DebugCollider;

#[derive(Component, Clone, Debug)]
pub enum Collider {
    Cube { size: Vec3 },
    Sphere { radius: f32 },
    Capsule { a: Vec3, b: Vec3, radius: f32 },
}

impl Collider {
    pub const fn cube(size: Vec3) -> Self {
        Self::Cube { size }
    }

    pub const fn sphere(radius: f32) -> Self {
        Self::Sphere { radius }
    }

    pub const fn capsule(a: Vec3, b: Vec3, radius: f32) -> Self {
        Self::Capsule { a, b, radius }
    }

    pub fn to_shape(&self) -> SharedShape {
        match *self {
            Self::Cube { size } => SharedShape::cuboid(size.x / 2.0, size.y / 2.0, size.z / 2.0),
            Self::Sphere { radius } => SharedShape::ball(radius),
            Self::Capsule { a, b, radius } => SharedShape::capsule(
                Point3::new(a.x, a.y, a.z),
                Point3::new(b.x, b.y, b.z),
                radius,
            ),
        }
    }
}
