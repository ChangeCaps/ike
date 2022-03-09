use std::ops::{Deref, DerefMut, Mul};

use ike_ecs::Component;
use ike_math::{const_vec3, Mat3, Mat4, Quat, Vec3};

#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub struct Transform {
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl Default for Transform {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl Transform {
    pub const IDENTITY: Self = Self {
        translation: Vec3::ZERO,
        rotation: Quat::IDENTITY,
        scale: Vec3::ONE,
    };

    pub const fn from_xyz(x: f32, y: f32, z: f32) -> Self {
        Self {
            translation: const_vec3!([x, y, z]),
            ..Self::IDENTITY
        }
    }

    pub const fn from_translation(translation: Vec3) -> Self {
        Self {
            translation,
            ..Self::IDENTITY
        }
    }

    pub const fn from_rotation(rotation: Quat) -> Self {
        Self {
            rotation,
            ..Self::IDENTITY
        }
    }

    pub const fn from_scale(scale: Vec3) -> Self {
        Self {
            scale,
            ..Self::IDENTITY
        }
    }

    pub const fn with_translation(mut self, translation: Vec3) -> Self {
        self.translation = translation;
        self
    }

    pub const fn with_rotation(mut self, rotation: Quat) -> Self {
        self.rotation = rotation;
        self
    }

    pub const fn with_scale(mut self, scale: Vec3) -> Self {
        self.scale = scale;
        self
    }

    pub fn local_x(&self) -> Vec3 {
        self.rotation * Vec3::X
    }

    pub fn local_y(&self) -> Vec3 {
        self.rotation * Vec3::Y
    }

    pub fn local_z(&self) -> Vec3 {
        self.rotation * Vec3::Z
    }

    pub fn matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.translation)
    }

    pub fn look_at(&mut self, target: Vec3, up: Vec3) {
        let forward = Vec3::normalize(self.translation - target);
        let right = up.cross(forward).normalize();
        let up = forward.cross(right);
        self.rotation = Quat::from_mat3(&Mat3::from_cols(right, up, forward));
    }
}

impl Mul<Vec3> for Transform {
    type Output = Vec3;

    fn mul(self, mut rhs: Vec3) -> Self::Output {
        rhs = self.scale * rhs;
        rhs = self.rotation * rhs;
        rhs = self.translation + rhs;
        rhs
    }
}

impl Mul<Transform> for Transform {
    type Output = Self;

    fn mul(self, rhs: Transform) -> Self::Output {
        Self {
            translation: self * rhs.translation,
            rotation: self.rotation * rhs.rotation,
            scale: self.scale * rhs.scale,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq)]
pub struct GlobalTransform(pub Transform);

impl GlobalTransform {
    pub const IDENTITY: Self = Self(Transform::IDENTITY);

    pub const fn new(transform: Transform) -> Self {
        Self(transform)
    }

    pub const fn transform(&self) -> Transform {
        self.0
    }

    pub fn matrix(&self) -> Mat4 {
        self.0.matrix()
    }
}

impl Deref for GlobalTransform {
    type Target = Transform;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for GlobalTransform {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<Transform> for GlobalTransform {
    fn from(transform: Transform) -> Self {
        Self::new(transform)
    }
}

impl Into<Transform> for GlobalTransform {
    fn into(self) -> Transform {
        *self
    }
}
