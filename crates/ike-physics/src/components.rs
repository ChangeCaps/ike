use deref_derive::Deref;
use ike_ecs::prelude::Component;
use ike_math::Vec3;

use crate::rapier::{dynamics, geometry};

#[derive(Component, Clone, Copy, Debug, Deref)]
pub struct RigidBodyHandle(pub(crate) dynamics::RigidBodyHandle);

#[derive(Component, Clone, Copy, Debug, Deref)]
pub struct ColliderHandle(pub(crate) geometry::ColliderHandle);

#[derive(Component, Clone, Copy, Debug, Default, PartialEq)]
pub struct Velocity {
    pub linear: Vec3,
    pub angular: Vec3,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq)]
pub enum RigidBody {
    #[default]
    Dynamic,
    Static,
    Kinematic,
}

impl RigidBody {
    #[inline]
    pub(crate) fn body_type(&self) -> dynamics::RigidBodyType {
        match self {
            Self::Dynamic => dynamics::RigidBodyType::Dynamic,
            Self::Static => dynamics::RigidBodyType::Fixed,
            Self::Kinematic => dynamics::RigidBodyType::KinematicPositionBased,
        }
    }

    #[inline]
    pub(crate) fn builder(&self) -> dynamics::RigidBodyBuilder {
        dynamics::RigidBodyBuilder::new(self.body_type())
    }
}

#[derive(Component, Clone, Debug, PartialEq)]
pub enum Collider {
    Sphere { radius: f32 },
    Capsule { radius: f32, half_height: f32 },
    Cuboid { half_extents: Vec3 },
    ConvexHull { points: Vec<Vec3> },
}

impl Collider {
    #[inline]
    pub const fn sphere(radius: f32) -> Self {
        Self::Sphere { radius }
    }

    #[inline]
    pub const fn capsule(radius: f32, half_height: f32) -> Self {
        Self::Capsule {
            radius,
            half_height,
        }
    }

    #[inline]
    pub const fn cuboid(half_extents: Vec3) -> Self {
        Self::Cuboid { half_extents }
    }

    #[inline]
    pub const fn convex_hull(points: Vec<Vec3>) -> Self {
        Self::ConvexHull { points }
    }

    #[inline]
    pub(crate) fn builder(&self) -> geometry::ColliderBuilder {
        match self {
            Collider::Sphere { radius } => geometry::ColliderBuilder::ball(*radius),
            Collider::Capsule {
                radius,
                half_height,
            } => geometry::ColliderBuilder::capsule_y(*radius, *half_height),
            Collider::Cuboid { half_extents } => {
                geometry::ColliderBuilder::cuboid(half_extents.x, half_extents.y, half_extents.z)
            }
            Collider::ConvexHull { points } => {
                let points = points
                    .iter()
                    .map(|p| nalgebra::Point::from_slice(&[p.x, p.y, p.z]))
                    .collect::<Vec<_>>();

                geometry::ColliderBuilder::convex_hull(&points).unwrap()
            }
        }
    }
}
